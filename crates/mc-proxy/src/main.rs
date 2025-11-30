use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{BigEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

fn read_varint_sync(cursor: &mut Cursor<&[u8]>) -> eyre::Result<i32> {
    let mut result = 0i32;
    let mut shift = 0;
    loop {
        let byte = ReadBytesExt::read_u8(cursor)?;
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

async fn read_varint(stream: &mut (impl AsyncReadExt + Unpin)) -> eyre::Result<i32> {
    let mut result = 0i32;
    let mut shift = 0;
    loop {
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;
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

fn varint_bytes(value: i32) -> Vec<u8> {
    let mut result = Vec::new();
    let mut val = value as u32;
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if val == 0 {
            break;
        }
    }
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum State {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PacketDirection {
    ClientToServer,
    ServerToClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordedPacket {
    timestamp_ms: u64,
    state: State,
    direction: PacketDirection,
    packet_id: i32,
    packet_name: String,
    raw_data: Vec<u8>,
}

#[derive(Default, Serialize, Deserialize)]
struct PacketRecording {
    start_time: u64,
    packets: Vec<RecordedPacket>,
}

fn decode_string(cursor: &mut Cursor<&[u8]>) -> eyre::Result<String> {
    let len = read_varint_sync(cursor)? as usize;
    let pos = cursor.position() as usize;
    let data = cursor.get_ref();
    if pos + len > data.len() {
        eyre::bail!("String too long");
    }
    let s = String::from_utf8_lossy(&data[pos..pos + len]).to_string();
    cursor.set_position((pos + len) as u64);
    Ok(s)
}

fn get_packet_name(state: State, direction: PacketDirection, packet_id: i32) -> &'static str {
    let dir = match direction {
        PacketDirection::ClientToServer => "C->S",
        PacketDirection::ServerToClient => "S->C",
    };
    match (state, dir, packet_id) {
        (State::Handshaking, "C->S", 0x00) => "Handshake",
        (State::Status, "C->S", 0x00) => "Status Request",
        (State::Status, "C->S", 0x01) => "Ping Request",
        (State::Status, "S->C", 0x00) => "Status Response",
        (State::Status, "S->C", 0x01) => "Ping Response",
        (State::Login, "C->S", 0x00) => "Login Start",
        (State::Login, "C->S", 0x03) => "Login Acknowledged",
        (State::Login, "S->C", 0x00) => "Disconnect",
        (State::Login, "S->C", 0x01) => "Encryption Request",
        (State::Login, "S->C", 0x02) => "Login Success",
        (State::Login, "S->C", 0x03) => "Set Compression",
        (State::Configuration, "C->S", 0x00) => "Client Information",
        (State::Configuration, "C->S", 0x02) => "Plugin Message",
        (State::Configuration, "C->S", 0x03) => "Finish Config Ack",
        (State::Configuration, "C->S", 0x07) => "Known Packs Response",
        (State::Configuration, "S->C", 0x03) => "Finish Configuration",
        (State::Configuration, "S->C", 0x07) => "Registry Data",
        (State::Configuration, "S->C", 0x0E) => "Known Packs",
        (State::Play, "S->C", 0x22) => "Game Event",
        (State::Play, "S->C", 0x23) => "Keep Alive",
        (State::Play, "S->C", 0x28) => "Login (Play)",
        (State::Play, "S->C", 0x2C) => "Chunk Data",
        (State::Play, "S->C", 0x40) => "Sync Player Position",
        (State::Play, "C->S", 0x00) => "Accept Teleportation",
        (State::Play, "C->S", 0x1A) => "Keep Alive Response",
        _ => "Unknown",
    }
}

async fn read_packet(stream: &mut (impl AsyncReadExt + Unpin)) -> eyre::Result<Vec<u8>> {
    let length = read_varint(stream).await?;
    let mut data = vec![0u8; length as usize];
    stream.read_exact(&mut data).await?;

    // Include length prefix in returned data
    let mut full = varint_bytes(length);
    full.extend_from_slice(&data);
    Ok(full)
}

async fn forward_packet(
    stream: &mut (impl AsyncWriteExt + Unpin),
    packet: &[u8],
) -> eyre::Result<()> {
    stream.write_all(packet).await?;
    stream.flush().await?;
    Ok(())
}

async fn proxy_connection(
    mut client: TcpStream,
    server_addr: &str,
    recording: Arc<Mutex<PacketRecording>>,
) -> eyre::Result<()> {
    let mut server = TcpStream::connect(server_addr).await?;
    info!("Connected to upstream server at {}", server_addr);

    let mut state = State::Handshaking;
    let mut packet_num = 0usize;

    let start_time = {
        let rec = recording.lock().unwrap();
        rec.start_time
    };

    loop {
        let mut client_buf = [0u8; 1];
        let mut server_buf = [0u8; 1];

        tokio::select! {
            biased;

            // Check for data from client
            result = client.peek(&mut client_buf) => {
                match result {
                    Ok(0) => {
                        info!("Client disconnected");
                        break;
                    }
                    Ok(_) => {
                        let packet = read_packet(&mut client).await?;

                        // Parse packet
                        let mut cursor = Cursor::new(&packet[..]);
                        let _length = read_varint_sync(&mut cursor)?;
                        let packet_id = read_varint_sync(&mut cursor)?;

                        let state_name = format!("{state:?}").to_lowercase();
                        let packet_name = get_packet_name(state, PacketDirection::ClientToServer, packet_id);

                        info!("#{} [{}] C->S 0x{:02X} {} ({} bytes)",
                            packet_num, state_name, packet_id, packet_name, packet.len());

                        // Record the packet
                        {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64;
                            let mut rec = recording.lock().unwrap();
                            rec.packets.push(RecordedPacket {
                                timestamp_ms: now - start_time,
                                state,
                                direction: PacketDirection::ClientToServer,
                                packet_id,
                                packet_name: packet_name.to_string(),
                                raw_data: packet.clone(),
                            });
                        }

                        // Handle state transitions
                        if matches!(state, State::Handshaking) && packet_id == 0 {
                            let _ = decode_string(&mut cursor); // server address
                            let _ = ReadBytesExt::read_u16::<BigEndian>(&mut cursor); // port
                            if let Ok(next) = read_varint_sync(&mut cursor) {
                                state = match next {
                                    1 => State::Status,
                                    2 => State::Login,
                                    _ => state,
                                };
                                info!("  -> State transition to {:?}", state);
                            }
                        }

                        if matches!(state, State::Login) && packet_id == 0x03 {
                            state = State::Configuration;
                            info!("  -> State transition to Configuration");
                        }

                        if matches!(state, State::Configuration) && packet_id == 0x03 {
                            state = State::Play;
                            info!("  -> State transition to Play");
                        }

                        forward_packet(&mut server, &packet).await?;
                        packet_num += 1;
                    }
                    Err(e) => {
                        warn!("Client peek error: {}", e);
                        break;
                    }
                }
            }

            // Check for data from server
            result = server.peek(&mut server_buf) => {
                match result {
                    Ok(0) => {
                        info!("Server disconnected");
                        break;
                    }
                    Ok(_) => {
                        let packet = read_packet(&mut server).await?;

                        // Parse packet
                        let mut cursor = Cursor::new(&packet[..]);
                        let _length = read_varint_sync(&mut cursor)?;
                        let packet_id = read_varint_sync(&mut cursor)?;

                        let state_name = format!("{state:?}").to_lowercase();
                        let packet_name = get_packet_name(state, PacketDirection::ServerToClient, packet_id);

                        info!("#{} [{}] S->C 0x{:02X} {} ({} bytes)",
                            packet_num, state_name, packet_id, packet_name, packet.len());

                        // Record the packet
                        {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64;
                            let mut rec = recording.lock().unwrap();
                            rec.packets.push(RecordedPacket {
                                timestamp_ms: now - start_time,
                                state,
                                direction: PacketDirection::ServerToClient,
                                packet_id,
                                packet_name: packet_name.to_string(),
                                raw_data: packet.clone(),
                            });
                        }

                        // Log extra info for registry data
                        if matches!(state, State::Configuration) && packet_id == 0x07 {
                            if let Ok(registry) = decode_string(&mut cursor) {
                                info!("  Registry: {}", registry);
                            }
                        }

                        forward_packet(&mut client, &packet).await?;
                        packet_num += 1;
                    }
                    Err(e) => {
                        warn!("Server peek error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn save_recording(recording: &PacketRecording, path: &str) -> eyre::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, recording)?;
    info!("Saved {} packets to {}", recording.packets.len(), path);
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_proxy=info".parse()?),
        )
        .init();

    let listen_port = std::env::args().nth(1).unwrap_or("25565".to_string());
    let target_port = std::env::args().nth(2).unwrap_or("25566".to_string());
    let output_file = std::env::args()
        .nth(3)
        .unwrap_or("recording.json".to_string());

    let listen_addr = format!("0.0.0.0:{listen_port}");
    let target_addr = format!("127.0.0.1:{target_port}");

    info!("MC Packet Capture Proxy");
    info!("Listen: {} -> Forward: {}", listen_addr, target_addr);
    info!("Recording to: {}", output_file);
    info!("");

    let listener = TcpListener::bind(&listen_addr).await?;

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let recording = Arc::new(Mutex::new(PacketRecording {
        start_time,
        packets: Vec::new(),
    }));

    // Save on Ctrl+C
    let recording_clone = recording.clone();
    let output_clone = output_file.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Interrupted, saving recording...");
        let rec = recording_clone.lock().unwrap();
        if let Err(e) = save_recording(&rec, &output_clone) {
            warn!("Failed to save recording: {}", e);
        }
        std::process::exit(0);
    });

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("=== New connection from {} ===", addr);

        let target = target_addr.clone();
        let recording = recording.clone();

        tokio::spawn(async move {
            if let Err(e) = proxy_connection(stream, &target, recording).await {
                warn!("Proxy error: {}", e);
            }
            info!("=== Connection closed ===");
        });
    }
}
