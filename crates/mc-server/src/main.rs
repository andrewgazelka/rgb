//! Minecraft server using Flecs ECS with REST API explorer
//!
//! This server uses the mc-server-lib crate for all ECS logic.
//! The async Tokio runtime handles network I/O, while Flecs handles game logic.
//!
//! Access the Flecs Explorer at: https://www.flecs.dev/explorer
//! (Connect to localhost:27750)

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use bytes::Bytes;
use mc_protocol::read_varint;
use mc_server_lib::{IncomingPacket, NetworkChannels, ServerConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Active connections map (connection_id -> sender for that connection)
type ConnectionMap = Arc<RwLock<HashMap<u64, tokio::sync::mpsc::Sender<Bytes>>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_server=debug".parse()?),
        )
        .init();

    // Create network channels for ECS <-> async bridge
    let channels = NetworkChannels::new();
    let config = ServerConfig::default();

    info!(
        "Starting Minecraft server with Flecs ECS (version {})",
        mc_data::PROTOCOL_NAME
    );
    info!(
        "Flecs Explorer available at https://www.flecs.dev/explorer (connect to localhost:{})",
        config.rest_port
    );

    // Spawn the ECS world in a blocking thread
    let ingress_tx = channels.ingress_tx.clone();
    let egress_rx = channels.egress_rx.clone();

    // Connection map for routing outgoing packets
    let connections: ConnectionMap = Arc::new(RwLock::new(HashMap::new()));
    let connections_clone = connections.clone();

    // Spawn ECS thread
    let _ecs_handle = std::thread::spawn(move || {
        let world = mc_server_lib::create_world(&channels);
        info!("Flecs world initialized with all modules");
        mc_server_lib::run_with_explorer(&world, &config)
    });

    // Spawn egress handler (routes packets from ECS to connections)
    // Note: crossbeam recv() is blocking, so we use a separate thread
    let rt_handle = tokio::runtime::Handle::current();
    let _egress_handle = std::thread::spawn(move || {
        while let Ok(packet) = egress_rx.recv() {
            debug!(
                "Async egress: routing packet to conn_id={}, len={}",
                packet.connection_id,
                packet.data.len()
            );
            let connections = connections_clone.clone();
            let data = packet.data;
            let conn_id = packet.connection_id;
            rt_handle.spawn(async move {
                let connections = connections.read().await;
                if let Some(tx) = connections.get(&conn_id) {
                    debug!("Async egress: found connection, sending via mpsc");
                    match tx.send(data).await {
                        Ok(()) => debug!("Async egress: mpsc send succeeded"),
                        Err(e) => debug!("Async egress: mpsc send failed: {}", e),
                    }
                } else {
                    debug!("Async egress: connection {} not found!", conn_id);
                }
            });
        }
    });

    // Start TCP listener
    let addr = "0.0.0.0:25565";
    let listener = TcpListener::bind(addr).await?;
    info!("Minecraft server listening on {}", addr);

    // Connection ID counter
    let mut next_conn_id: u64 = 1;

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Connection from {}", addr);

        let conn_id = next_conn_id;
        next_conn_id += 1;

        let ingress_tx = ingress_tx.clone();
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

            if let Err(e) = result {
                debug!("Connection {} closed: {}", conn_id, e);
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    conn_id: u64,
    ingress_tx: crossbeam_channel::Sender<IncomingPacket>,
    mut egress_rx: tokio::sync::mpsc::Receiver<Bytes>,
) -> anyhow::Result<()> {
    let (mut reader, mut writer) = stream.into_split();

    // Spawn writer task
    let writer_handle = tokio::spawn(async move {
        while let Some(data) = egress_rx.recv().await {
            debug!("Writer task: writing {} bytes to socket", data.len());
            if writer.write_all(&data).await.is_err() {
                debug!("Writer task: write failed");
                break;
            }
            if writer.flush().await.is_err() {
                debug!("Writer task: flush failed");
                break;
            }
            debug!("Writer task: write+flush complete");
        }
        debug!("Writer task: channel closed, exiting");
    });

    // Read packets and send to ECS
    loop {
        // Read packet length
        let Ok(length) = read_varint_async(&mut reader).await else {
            break;
        };

        if length <= 0 {
            continue;
        }

        // Read packet data
        let mut data = vec![0u8; length as usize];
        if reader.read_exact(&mut data).await.is_err() {
            break;
        }

        // Parse packet ID
        let mut cursor = Cursor::new(&data);
        let packet_id = read_varint(&mut cursor)?;
        let remaining = data[cursor.position() as usize..].to_vec();

        // Send to ECS
        let _ = ingress_tx.send(IncomingPacket {
            connection_id: conn_id,
            packet_id,
            data: remaining.into(),
        });
    }

    writer_handle.abort();
    Ok(())
}

async fn read_varint_async<R: AsyncReadExt + Unpin>(reader: &mut R) -> anyhow::Result<i32> {
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
            anyhow::bail!("VarInt too large");
        }
    }
    Ok(result)
}
