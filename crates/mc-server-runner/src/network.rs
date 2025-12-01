//! Network layer - async TCP server bridging to ECS

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::thread;

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use mc_protocol::read_varint;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::components::{DisconnectEvent, IncomingPacket, OutgoingPacket};

/// Active connections map (connection_id -> sender for that connection)
type ConnectionMap = Arc<RwLock<HashMap<u64, tokio::sync::mpsc::Sender<Bytes>>>>;

/// Channels for network I/O between async Tokio runtime and sync ECS world
pub struct NetworkChannels {
    /// Sender for incoming packets (async -> ECS)
    pub ingress_tx: Sender<IncomingPacket>,
    /// Receiver for incoming packets (async -> ECS)
    pub ingress_rx: Receiver<IncomingPacket>,
    /// Sender for outgoing packets (ECS -> async)
    pub egress_tx: Sender<OutgoingPacket>,
    /// Receiver for outgoing packets (ECS -> async)
    pub egress_rx: Receiver<OutgoingPacket>,
    /// Sender for disconnect events (async -> ECS)
    pub disconnect_tx: Sender<DisconnectEvent>,
    /// Receiver for disconnect events (async -> ECS)
    pub disconnect_rx: Receiver<DisconnectEvent>,
}

impl NetworkChannels {
    #[must_use]
    pub fn new() -> Self {
        let (ingress_tx, ingress_rx) = crossbeam_channel::unbounded();
        let (egress_tx, egress_rx) = crossbeam_channel::unbounded();
        let (disconnect_tx, disconnect_rx) = crossbeam_channel::unbounded();
        Self {
            ingress_tx,
            ingress_rx,
            egress_tx,
            egress_rx,
            disconnect_tx,
            disconnect_rx,
        }
    }
}

impl Default for NetworkChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the network thread with async TCP server
pub fn start_network_thread(
    ingress_tx: Sender<IncomingPacket>,
    egress_rx: Receiver<OutgoingPacket>,
    disconnect_tx: Sender<DisconnectEvent>,
) {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        rt.block_on(async move {
            if let Err(e) = run_network(ingress_tx, egress_rx, disconnect_tx).await {
                error!("Network error: {}", e);
            }
        });
    });

    info!("Network thread started - TCP server starting on port 25565");
}

async fn run_network(
    ingress_tx: Sender<IncomingPacket>,
    egress_rx: Receiver<OutgoingPacket>,
    disconnect_tx: Sender<DisconnectEvent>,
) -> eyre::Result<()> {
    // Connection map for routing outgoing packets
    let connections: ConnectionMap = Arc::new(RwLock::new(HashMap::new()));

    // Spawn egress handler (routes packets from ECS to connections)
    let connections_for_egress = connections.clone();
    tokio::spawn(async move {
        loop {
            let egress_rx = egress_rx.clone();
            let connections = connections_for_egress.clone();

            let packet = tokio::task::spawn_blocking(move || egress_rx.recv())
                .await
                .ok()
                .and_then(|r| r.ok());

            let Some(packet) = packet else {
                break;
            };

            let conn_id = packet.connection_id;
            let data = packet.data;

            let conns = connections.read().await;
            if let Some(tx) = conns.get(&conn_id) {
                let _ = tx.send(data).await;
            }
        }
    });

    // Start TCP listener
    let port: u16 = std::env::var("MC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(25565);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    let actual_port = listener.local_addr()?.port();

    info!("Minecraft server listening on 0.0.0.0:{}", actual_port);

    let mut next_conn_id: u64 = 1;

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Connection from {}", addr);

        let conn_id = next_conn_id;
        next_conn_id += 1;

        let ingress_tx = ingress_tx.clone();
        let disconnect_tx = disconnect_tx.clone();
        let connections = connections.clone();

        tokio::spawn(async move {
            // Create channel for this connection's outgoing packets
            let (tx, rx) = tokio::sync::mpsc::channel::<Bytes>(256);

            // Register connection
            {
                let mut conns = connections.write().await;
                conns.insert(conn_id, tx);
            }

            // Handle connection
            let result = handle_connection(stream, conn_id, ingress_tx, rx).await;

            // Unregister connection
            {
                let mut conns = connections.write().await;
                conns.remove(&conn_id);
            }

            // Notify ECS of disconnection
            info!("Connection {} disconnected", conn_id);
            let _ = disconnect_tx.send(DisconnectEvent {
                connection_id: conn_id,
            });

            if let Err(e) = result {
                debug!("Connection {} closed: {}", conn_id, e);
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    conn_id: u64,
    ingress_tx: Sender<IncomingPacket>,
    mut egress_rx: tokio::sync::mpsc::Receiver<Bytes>,
) -> eyre::Result<()> {
    let (mut reader, mut writer) = stream.into_split();

    // Spawn writer task
    let writer_handle = tokio::spawn(async move {
        while let Some(data) = egress_rx.recv().await {
            if writer.write_all(&data).await.is_err() {
                break;
            }
            if writer.flush().await.is_err() {
                break;
            }
        }
    });

    // Read packets and send to ECS
    loop {
        let Ok(length) = read_varint_async(&mut reader).await else {
            break;
        };

        if length <= 0 {
            continue;
        }

        let mut data = vec![0u8; length as usize];
        if reader.read_exact(&mut data).await.is_err() {
            break;
        }

        let mut cursor = Cursor::new(&data);
        let Ok(packet_id) = read_varint(&mut cursor) else {
            break;
        };
        let remaining = data[cursor.position() as usize..].to_vec();

        let _ = ingress_tx.send(IncomingPacket {
            connection_id: conn_id,
            packet_id,
            data: remaining.into(),
        });
    }

    writer_handle.abort();
    Ok(())
}

async fn read_varint_async<R: AsyncReadExt + Unpin>(reader: &mut R) -> eyre::Result<i32> {
    let mut result = 0i32;
    let mut shift = 0;
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf).await?;
        let byte = buf[0];
        result |= ((byte & 0x7F) as i32) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            eyre::bail!("VarInt too large");
        }
    }
    Ok(result)
}
