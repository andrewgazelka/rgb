use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};
use mc_protocol::{read_varint, write_varint, Decode, Encode};
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    Handshaking,
    Status,
    Login,
    #[allow(dead_code)]
    Play,
}

#[derive(Debug, Serialize)]
struct ServerStatus {
    version: Version,
    players: Players,
    description: Description,
    #[serde(rename = "enforcesSecureChat")]
    enforces_secure_chat: bool,
}

#[derive(Debug, Serialize)]
struct Version {
    name: String,
    protocol: i32,
}

#[derive(Debug, Serialize)]
struct Players {
    max: i32,
    online: i32,
    sample: Vec<PlayerSample>,
}

#[derive(Debug, Serialize)]
struct PlayerSample {
    name: String,
    id: String,
}

#[derive(Debug, Serialize)]
struct Description {
    text: String,
}

async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut state = ConnectionState::Handshaking;

    loop {
        // Read packet length (VarInt)
        let length = read_varint_async(&mut stream).await?;
        if length == 0 {
            continue;
        }

        // Read the entire packet
        let mut packet_data = vec![0u8; length as usize];
        stream.read_exact(&mut packet_data).await?;

        let mut cursor = Cursor::new(&packet_data);
        let packet_id = read_varint(&mut cursor)?;

        match state {
            ConnectionState::Handshaking => {
                if packet_id == 0 {
                    // Handshake packet
                    let protocol_version = read_varint(&mut cursor)?;
                    let _server_address = String::decode(&mut cursor)?;
                    let _server_port = ReadBytesExt::read_u16::<BigEndian>(&mut cursor)?;
                    let next_state = read_varint(&mut cursor)?;

                    info!(
                        "Handshake: protocol={}, next_state={}",
                        protocol_version, next_state
                    );

                    state = match next_state {
                        1 => ConnectionState::Status,
                        2 => ConnectionState::Login,
                        _ => {
                            warn!("Unknown next state: {}", next_state);
                            break;
                        }
                    };
                }
            }
            ConnectionState::Status => {
                match packet_id {
                    0 => {
                        // Status Request
                        info!("Status request received");

                        let status = ServerStatus {
                            version: Version {
                                name: mc_packets::PROTOCOL_NAME.to_string(),
                                protocol: mc_packets::PROTOCOL_VERSION,
                            },
                            players: Players {
                                max: 100,
                                online: 0,
                                sample: vec![],
                            },
                            description: Description {
                                text: "A Rust Minecraft Server".to_string(),
                            },
                            enforces_secure_chat: false,
                        };

                        let json = serde_json::to_string(&status)?;
                        send_status_response(&mut stream, &json).await?;
                    }
                    1 => {
                        // Ping Request
                        let time = i64::decode(&mut cursor)?;
                        info!("Ping request received: time={}", time);

                        // Send Pong Response
                        send_pong_response(&mut stream, time).await?;

                        // Close connection after ping/pong
                        break;
                    }
                    _ => {
                        warn!("Unknown status packet: {}", packet_id);
                    }
                }
            }
            ConnectionState::Login => {
                if packet_id == 0 {
                    // Login Start
                    let name = String::decode(&mut cursor)?;
                    info!("Login attempt from: {}", name);

                    // For now, just disconnect
                    send_disconnect(&mut stream, "Server not implemented yet").await?;
                    break;
                }
            }
            ConnectionState::Play => {
                // Not implemented
                break;
            }
        }
    }

    Ok(())
}

async fn read_varint_async(stream: &mut TcpStream) -> anyhow::Result<i32> {
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
            anyhow::bail!("VarInt too large");
        }
    }
    Ok(result)
}

fn encode_varint(value: i32) -> Vec<u8> {
    let mut buf = Vec::new();
    write_varint(&mut buf, value).unwrap();
    buf
}

async fn send_packet(stream: &mut TcpStream, packet_id: i32, data: &[u8]) -> anyhow::Result<()> {
    let packet_id_bytes = encode_varint(packet_id);
    let length = packet_id_bytes.len() + data.len();
    let length_bytes = encode_varint(length as i32);

    stream.write_all(&length_bytes).await?;
    stream.write_all(&packet_id_bytes).await?;
    stream.write_all(data).await?;
    stream.flush().await?;

    Ok(())
}

async fn send_status_response(stream: &mut TcpStream, json: &str) -> anyhow::Result<()> {
    let mut data = Vec::new();
    json.encode(&mut data)?;
    send_packet(stream, 0, &data).await
}

async fn send_pong_response(stream: &mut TcpStream, time: i64) -> anyhow::Result<()> {
    let mut data = Vec::new();
    time.encode(&mut data)?;
    send_packet(stream, 1, &data).await
}

async fn send_disconnect(stream: &mut TcpStream, reason: &str) -> anyhow::Result<()> {
    // Login disconnect packet (ID 0)
    // The reason is a JSON Chat component
    let json = serde_json::json!({"text": reason}).to_string();
    let mut data = Vec::new();
    json.encode(&mut data)?;
    send_packet(stream, 0, &data).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_server=info".parse()?),
        )
        .init();

    let addr = "0.0.0.0:25565";
    let listener = TcpListener::bind(addr).await?;
    info!("Minecraft server listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                warn!("Connection error: {}", e);
            }
        });
    }
}
