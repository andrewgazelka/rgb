use std::fs::File;
use std::io::{BufWriter, Cursor, Read as IoRead};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use mc_protocol::{Decode, Encode, Uuid, read_varint, write_varint};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordedPacket {
    timestamp_ms: u64,
    state: ConnectionState,
    direction: PacketDirection,
    packet_id: i32,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PacketDirection {
    Serverbound,
    Clientbound,
}

struct Client {
    stream: TcpStream,
    state: ConnectionState,
    player_name: String,
    player_uuid: u128,
    recorded_packets: Vec<RecordedPacket>,
    start_time: u64,
    compression_threshold: Option<i32>,
}

impl Client {
    fn new(stream: TcpStream, player_name: String) -> Self {
        let player_uuid = offline_uuid(&player_name);
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            stream,
            state: ConnectionState::Handshaking,
            player_name,
            player_uuid,
            recorded_packets: Vec::new(),
            start_time,
            compression_threshold: None,
        }
    }

    fn record_packet(&mut self, direction: PacketDirection, packet_id: i32, data: &[u8]) {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            - self.start_time;

        self.recorded_packets.push(RecordedPacket {
            timestamp_ms,
            state: self.state,
            direction,
            packet_id,
            data: data.to_vec(),
        });
    }

    async fn read_varint(&mut self) -> eyre::Result<i32> {
        let mut result = 0i32;
        let mut shift = 0;
        loop {
            let mut buf = [0u8; 1];
            self.stream.read_exact(&mut buf).await?;
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

    async fn read_packet(&mut self) -> eyre::Result<(i32, Vec<u8>)> {
        let length = self.read_varint().await?;
        if length == 0 {
            return Ok((-1, vec![]));
        }

        let mut data = vec![0u8; length as usize];
        self.stream.read_exact(&mut data).await?;

        // Handle compression
        let (packet_id, remaining) = if let Some(_threshold) = self.compression_threshold {
            let mut cursor = Cursor::new(&data);
            let data_length = read_varint(&mut cursor)?;

            if data_length == 0 {
                // Uncompressed packet
                let packet_id = read_varint(&mut cursor)?;
                let remaining = data[cursor.position() as usize..].to_vec();
                (packet_id, remaining)
            } else {
                // Compressed packet
                let compressed_data = &data[cursor.position() as usize..];
                let mut decoder = ZlibDecoder::new(compressed_data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;

                let mut cursor = Cursor::new(&decompressed);
                let packet_id = read_varint(&mut cursor)?;
                let remaining = decompressed[cursor.position() as usize..].to_vec();
                (packet_id, remaining)
            }
        } else {
            let mut cursor = Cursor::new(&data);
            let packet_id = read_varint(&mut cursor)?;
            let remaining = data[cursor.position() as usize..].to_vec();
            (packet_id, remaining)
        };

        // Record the packet
        self.record_packet(PacketDirection::Clientbound, packet_id, &remaining);

        Ok((packet_id, remaining))
    }

    async fn send_packet(&mut self, packet_id: i32, data: &[u8]) -> eyre::Result<()> {
        // Record the packet
        self.record_packet(PacketDirection::Serverbound, packet_id, data);

        let mut packet_id_bytes = Vec::new();
        write_varint(&mut packet_id_bytes, packet_id)?;

        if let Some(threshold) = self.compression_threshold {
            // Compression enabled
            let uncompressed_len = packet_id_bytes.len() + data.len();

            if uncompressed_len >= threshold as usize {
                // Compress the packet
                let mut uncompressed = packet_id_bytes.clone();
                uncompressed.extend_from_slice(data);

                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                std::io::Write::write_all(&mut encoder, &uncompressed)?;
                let compressed = encoder.finish()?;

                let mut data_length_bytes = Vec::new();
                write_varint(&mut data_length_bytes, uncompressed_len as i32)?;

                let length = data_length_bytes.len() + compressed.len();
                let mut length_bytes = Vec::new();
                write_varint(&mut length_bytes, length as i32)?;

                self.stream.write_all(&length_bytes).await?;
                self.stream.write_all(&data_length_bytes).await?;
                self.stream.write_all(&compressed).await?;
            } else {
                // Send uncompressed (data_length = 0)
                let mut data_length_bytes = Vec::new();
                write_varint(&mut data_length_bytes, 0)?;

                let length = data_length_bytes.len() + packet_id_bytes.len() + data.len();
                let mut length_bytes = Vec::new();
                write_varint(&mut length_bytes, length as i32)?;

                self.stream.write_all(&length_bytes).await?;
                self.stream.write_all(&data_length_bytes).await?;
                self.stream.write_all(&packet_id_bytes).await?;
                self.stream.write_all(data).await?;
            }
        } else {
            // No compression
            let length = packet_id_bytes.len() + data.len();
            let mut length_bytes = Vec::new();
            write_varint(&mut length_bytes, length as i32)?;

            self.stream.write_all(&length_bytes).await?;
            self.stream.write_all(&packet_id_bytes).await?;
            self.stream.write_all(data).await?;
        }

        self.stream.flush().await?;
        Ok(())
    }

    async fn connect(&mut self, host: &str, port: u16) -> eyre::Result<()> {
        // Send Handshake
        self.send_handshake(host, port).await?;

        // Send Login Start
        self.send_login_start().await?;

        // Handle login/configuration/play
        self.handle_login_phase().await?;

        Ok(())
    }

    async fn send_handshake(&mut self, host: &str, port: u16) -> eyre::Result<()> {
        let mut data = Vec::new();

        // Protocol Version (VarInt)
        write_varint(&mut data, mc_data::PROTOCOL_VERSION)?;

        // Server Address (String)
        host.encode(&mut data)?;

        // Server Port (Unsigned Short)
        WriteBytesExt::write_u16::<BigEndian>(&mut data, port)?;

        // Next State (VarInt) - 2 for Login
        write_varint(&mut data, 2)?;

        self.send_packet(0, &data).await?;
        self.state = ConnectionState::Login;
        info!("Sent Handshake (protocol {})", mc_data::PROTOCOL_VERSION);
        Ok(())
    }

    async fn send_login_start(&mut self) -> eyre::Result<()> {
        let mut data = Vec::new();

        // Player Name (String)
        self.player_name.encode(&mut data)?;

        // Player UUID
        Uuid(self.player_uuid).encode(&mut data)?;

        self.send_packet(0, &data).await?;
        info!(
            "Sent Login Start (name: {}, uuid: {:032x})",
            self.player_name, self.player_uuid
        );
        Ok(())
    }

    async fn handle_login_phase(&mut self) -> eyre::Result<()> {
        loop {
            let (packet_id, data) = self.read_packet().await?;
            if packet_id == -1 {
                continue;
            }

            let mut cursor = Cursor::new(&data);

            match self.state {
                ConnectionState::Login => {
                    if !self.handle_login_packet(packet_id, &mut cursor).await? {
                        break;
                    }
                }
                ConnectionState::Configuration => {
                    if !self
                        .handle_configuration_packet(packet_id, &mut cursor)
                        .await?
                    {
                        break;
                    }
                }
                ConnectionState::Play => {
                    if !self.handle_play_packet(packet_id, &mut cursor).await? {
                        break;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_login_packet(
        &mut self,
        packet_id: i32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> eyre::Result<bool> {
        match packet_id {
            0 => {
                // Disconnect
                let reason = String::decode(cursor)?;
                warn!("Disconnected during login: {}", reason);
                return Ok(false);
            }
            2 => {
                // Login Success
                let uuid = Uuid::decode(cursor)?;
                let name = String::decode(cursor)?;
                let properties_count = read_varint(cursor)?;
                info!(
                    "Login Success: {} (uuid: {:032x}, {} properties)",
                    name, uuid.0, properties_count
                );

                // Send Login Acknowledged
                self.send_packet(3, &[]).await?;
                self.state = ConnectionState::Configuration;
                info!("Sent Login Acknowledged, transitioning to Configuration");
            }
            3 => {
                // Set Compression
                let threshold = read_varint(cursor)?;
                info!("Set Compression threshold: {}", threshold);
                self.compression_threshold = Some(threshold);
            }
            _ => {
                debug!("Unknown login packet: 0x{:02X}", packet_id);
            }
        }
        Ok(true)
    }

    async fn handle_configuration_packet(
        &mut self,
        packet_id: i32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> eyre::Result<bool> {
        match packet_id {
            0 => {
                // Cookie Request
                let key = String::decode(cursor)?;
                debug!("Cookie Request: {}", key);
            }
            1 => {
                // Custom Payload (plugin message)
                let channel = String::decode(cursor)?;
                debug!("Custom Payload: channel={}", channel);

                // Respond with brand
                if channel == "minecraft:brand" {
                    let mut data = Vec::new();
                    "minecraft:brand".encode(&mut data)?;
                    // Brand name
                    "rust-client".encode(&mut data)?;
                    self.send_packet(2, &data).await?; // Serverbound Custom Payload
                    debug!("Sent brand response");
                }
            }
            2 => {
                // Disconnect
                let reason = String::decode(cursor)?;
                warn!("Disconnected during configuration: {}", reason);
                return Ok(false);
            }
            3 => {
                // Finish Configuration
                info!("Got Finish Configuration");
                self.send_packet(3, &[]).await?; // Finish Configuration Acknowledge
                self.state = ConnectionState::Play;
                info!("Transitioning to Play state");
            }
            7 => {
                // Registry Data
                let registry = String::decode(cursor)?;
                let count = read_varint(cursor)?;
                debug!("Registry Data: {} ({} entries)", registry, count);
            }
            12 => {
                // Update Enabled Features
                let count = read_varint(cursor)?;
                debug!("Update Enabled Features: {} features", count);
            }
            13 => {
                // Update Tags
                debug!("Update Tags");
            }
            14 => {
                // Known Packs
                let count = read_varint(cursor)?;
                debug!("Known Packs: {} packs", count);

                // Respond with empty known packs (accept defaults)
                let mut data = Vec::new();
                write_varint(&mut data, 0)?; // 0 known packs
                self.send_packet(7, &data).await?;
                debug!("Sent Known Packs response");
            }
            _ => {
                debug!(
                    "Configuration packet: 0x{:02X} ({} bytes)",
                    packet_id,
                    cursor.get_ref().len()
                );
            }
        }
        Ok(true)
    }

    async fn handle_play_packet(
        &mut self,
        packet_id: i32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> eyre::Result<bool> {
        match packet_id {
            0x26 => {
                // Game Event - packet ID 38
                let event = ReadBytesExt::read_u8(cursor)?;
                let value = ReadBytesExt::read_f32::<BigEndian>(cursor)?;
                debug!("Game Event: {} (value: {})", event, value);
            }
            0x2B => {
                // Keep Alive - packet ID 43
                let id = i64::decode(cursor)?;
                debug!("Keep Alive: {}", id);

                // Respond with same ID (serverbound keep alive is 0x1B = 27)
                let mut data = Vec::new();
                WriteBytesExt::write_i64::<BigEndian>(&mut data, id)?;
                self.send_packet(0x1B, &data).await?;
            }
            0x30 => {
                // Login (Play) - packet ID 48
                let entity_id = i32::decode(cursor)?;
                info!("Play Login: entity_id={}", entity_id);
            }
            0x2C => {
                // Level Chunk With Light - packet ID 44
                let chunk_x = i32::decode(cursor)?;
                let chunk_z = i32::decode(cursor)?;
                debug!("Chunk: ({}, {})", chunk_x, chunk_z);
            }
            0x46 => {
                // Player Position - packet ID 70
                let teleport_id = read_varint(cursor)?;
                debug!("Player Position: teleport_id={}", teleport_id);

                // Accept teleportation (serverbound is 0x00)
                let mut data = Vec::new();
                write_varint(&mut data, teleport_id)?;
                self.send_packet(0x00, &data).await?;
            }
            _ => {
                debug!(
                    "Play packet: 0x{:02X} ({} bytes)",
                    packet_id,
                    cursor.get_ref().len()
                );
            }
        }
        Ok(true)
    }

    fn save_recording(&self, path: &PathBuf) -> eyre::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.recorded_packets)?;
        info!(
            "Saved {} packets to {:?}",
            self.recorded_packets.len(),
            path
        );
        Ok(())
    }
}

fn offline_uuid(name: &str) -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let input = format!("OfflinePlayer:{}", name);
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash1 = hasher.finish();
    input.hash(&mut hasher);
    let hash2 = hasher.finish();

    let mut uuid = ((hash1 as u128) << 64) | (hash2 as u128);
    uuid = (uuid & 0xFFFFFFFFFFFF0FFFFFFFFFFFFFFF) | 0x00000000000030000000000000000000;
    uuid = (uuid & 0xFFFFFFFFFFFFFFFF3FFFFFFFFFFFFFFF) | 0x00000000000000008000000000000000;
    uuid
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_client=info".parse()?),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    let host = args.get(1).map(|s| s.as_str()).unwrap_or("127.0.0.1");
    let port: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(25565);
    let player_name = args
        .get(3)
        .cloned()
        .unwrap_or_else(|| "RustBot".to_string());
    let output_file = args
        .get(4)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("packets.json"));

    info!("MC Client - Connecting to {}:{}", host, port);
    info!("Player: {}", player_name);
    info!("Output: {:?}", output_file);

    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(&addr).await?;
    info!("Connected to {}", addr);

    let mut client = Client::new(stream, player_name);

    // Set up Ctrl+C handler to save on exit
    tokio::select! {
        result = client.connect(host, port) => {
            if let Err(e) = result {
                warn!("Connection error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Interrupted, saving recording...");
        }
    }

    client.save_recording(&output_file)?;

    Ok(())
}
