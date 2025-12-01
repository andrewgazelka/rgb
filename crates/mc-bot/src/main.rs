use std::io::{Cursor, Read as IoRead};
use std::time::Duration;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use mc_protocol::{Decode, Encode, Uuid, read_varint, write_varint};
use rand::Rng as _;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    Handshaking,
    Login,
    Configuration,
    Play,
}

struct Bot {
    stream: TcpStream,
    state: ConnectionState,
    player_name: String,
    player_uuid: u128,
    compression_threshold: Option<i32>,
    // Position tracking
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
    // Jump state
    velocity_y: f64,
    last_tick: std::time::Instant,
}

// Packet IDs for serverbound play packets
const ACCEPT_TELEPORTATION: i32 = 0x00;
const KEEP_ALIVE: i32 = 0x1B;
const MOVE_PLAYER_POS: i32 = 0x1D;
const MOVE_PLAYER_POS_ROT: i32 = 0x1E;
const MOVE_PLAYER_STATUS_ONLY: i32 = 0x20;
const CHUNK_BATCH_RECEIVED: i32 = 0x0A;

// Clientbound packet IDs
const CB_KEEP_ALIVE: i32 = 0x2B;
const CB_PLAYER_POSITION: i32 = 0x46;
const CB_LOGIN: i32 = 0x30;
const CB_CHUNK_BATCH_FINISHED: i32 = 0x0B;

impl Bot {
    fn new(stream: TcpStream, player_name: String) -> Self {
        let player_uuid = offline_uuid(&player_name);
        Self {
            stream,
            state: ConnectionState::Handshaking,
            player_name,
            player_uuid,
            compression_threshold: None,
            x: 0.0,
            y: 64.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: true,
            velocity_y: 0.0,
            last_tick: std::time::Instant::now(),
        }
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

        let (packet_id, remaining) = if let Some(_threshold) = self.compression_threshold {
            let mut cursor = Cursor::new(&data);
            let data_length = read_varint(&mut cursor)?;

            if data_length == 0 {
                let packet_id = read_varint(&mut cursor)?;
                let remaining = data[cursor.position() as usize..].to_vec();
                (packet_id, remaining)
            } else {
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

        Ok((packet_id, remaining))
    }

    async fn send_packet(&mut self, packet_id: i32, data: &[u8]) -> eyre::Result<()> {
        let mut packet_id_bytes = Vec::new();
        write_varint(&mut packet_id_bytes, packet_id)?;

        if let Some(threshold) = self.compression_threshold {
            let uncompressed_len = packet_id_bytes.len() + data.len();

            if uncompressed_len >= threshold as usize {
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

    async fn send_handshake(&mut self, host: &str, port: u16) -> eyre::Result<()> {
        let mut data = Vec::new();
        write_varint(&mut data, mc_data::PROTOCOL_VERSION)?;
        host.encode(&mut data)?;
        WriteBytesExt::write_u16::<BigEndian>(&mut data, port)?;
        write_varint(&mut data, 2)?; // Next state: Login

        self.send_packet(0, &data).await?;
        self.state = ConnectionState::Login;
        info!("Sent Handshake");
        Ok(())
    }

    async fn send_login_start(&mut self) -> eyre::Result<()> {
        let mut data = Vec::new();
        self.player_name.encode(&mut data)?;
        Uuid(self.player_uuid).encode(&mut data)?;

        self.send_packet(0, &data).await?;
        info!("Sent Login Start (name: {})", self.player_name);
        Ok(())
    }

    async fn send_position(&mut self) -> eyre::Result<()> {
        let mut data = Vec::new();
        WriteBytesExt::write_f64::<BigEndian>(&mut data, self.x)?;
        WriteBytesExt::write_f64::<BigEndian>(&mut data, self.y)?;
        WriteBytesExt::write_f64::<BigEndian>(&mut data, self.z)?;
        data.push(if self.on_ground { 1 } else { 0 });

        self.send_packet(MOVE_PLAYER_POS, &data).await?;
        Ok(())
    }

    async fn send_position_rotation(&mut self) -> eyre::Result<()> {
        let mut data = Vec::new();
        WriteBytesExt::write_f64::<BigEndian>(&mut data, self.x)?;
        WriteBytesExt::write_f64::<BigEndian>(&mut data, self.y)?;
        WriteBytesExt::write_f64::<BigEndian>(&mut data, self.z)?;
        WriteBytesExt::write_f32::<BigEndian>(&mut data, self.yaw)?;
        WriteBytesExt::write_f32::<BigEndian>(&mut data, self.pitch)?;
        data.push(if self.on_ground { 1 } else { 0 });

        self.send_packet(MOVE_PLAYER_POS_ROT, &data).await?;
        Ok(())
    }

    fn tick_physics(&mut self) {
        const GRAVITY: f64 = 32.0; // blocks/s^2
        const JUMP_VELOCITY: f64 = 9.0; // blocks/s
        const GROUND_Y: f64 = 64.0;

        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64();
        self.last_tick = now;

        if self.on_ground {
            // Jump!
            self.velocity_y = JUMP_VELOCITY;
            self.on_ground = false;
        } else {
            // Apply gravity
            self.velocity_y -= GRAVITY * dt;
            self.y += self.velocity_y * dt;

            // Check ground collision
            if self.y <= GROUND_Y {
                self.y = GROUND_Y;
                self.on_ground = true;
                self.velocity_y = 0.0;
            }
        }
    }

    async fn handle_login_packet(
        &mut self,
        packet_id: i32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> eyre::Result<bool> {
        match packet_id {
            0 => {
                let reason = String::decode(cursor)?;
                warn!("Disconnected during login: {}", reason);
                return Ok(false);
            }
            2 => {
                let uuid = Uuid::decode(cursor)?;
                let name = String::decode(cursor)?;
                info!("Login Success: {} (uuid: {:032x})", name, uuid.0);
                self.send_packet(3, &[]).await?; // Login Acknowledged
                self.state = ConnectionState::Configuration;
            }
            3 => {
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
            1 => {
                let channel = String::decode(cursor)?;
                debug!("Custom Payload: channel={}", channel);
                if channel == "minecraft:brand" {
                    let mut data = Vec::new();
                    "minecraft:brand".encode(&mut data)?;
                    "rust-bot".encode(&mut data)?;
                    self.send_packet(2, &data).await?;
                }
            }
            2 => {
                let reason = String::decode(cursor)?;
                warn!("Disconnected during configuration: {}", reason);
                return Ok(false);
            }
            3 => {
                info!("Got Finish Configuration");
                self.send_packet(3, &[]).await?; // Finish Configuration Acknowledge
                self.state = ConnectionState::Play;
                info!("Transitioning to Play state");
            }
            14 => {
                let count = read_varint(cursor)?;
                debug!("Known Packs: {} packs", count);
                let mut data = Vec::new();
                write_varint(&mut data, 0)?;
                self.send_packet(7, &data).await?;
            }
            _ => {
                debug!("Configuration packet: 0x{:02X}", packet_id);
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
            CB_LOGIN => {
                let entity_id = i32::decode(cursor)?;
                info!("Play Login: entity_id={}", entity_id);
            }
            CB_KEEP_ALIVE => {
                let id = i64::decode(cursor)?;
                debug!("Keep Alive: {}", id);
                let mut data = Vec::new();
                WriteBytesExt::write_i64::<BigEndian>(&mut data, id)?;
                self.send_packet(KEEP_ALIVE, &data).await?;
            }
            CB_PLAYER_POSITION => {
                // Parse position packet
                let teleport_id = read_varint(cursor)?;
                let x = ReadBytesExt::read_f64::<BigEndian>(cursor)?;
                let y = ReadBytesExt::read_f64::<BigEndian>(cursor)?;
                let z = ReadBytesExt::read_f64::<BigEndian>(cursor)?;
                // There are more fields but we just need position

                self.x = x;
                self.y = y;
                self.z = z;
                info!("Teleported to ({:.2}, {:.2}, {:.2})", x, y, z);

                // Accept teleportation
                let mut data = Vec::new();
                write_varint(&mut data, teleport_id)?;
                self.send_packet(ACCEPT_TELEPORTATION, &data).await?;
            }
            CB_CHUNK_BATCH_FINISHED => {
                // Respond to chunk batch
                let mut data = Vec::new();
                WriteBytesExt::write_f32::<BigEndian>(&mut data, 20.0)?; // chunks per tick
                self.send_packet(CHUNK_BATCH_RECEIVED, &data).await?;
            }
            _ => {
                // Silently ignore other packets
            }
        }
        Ok(true)
    }

    async fn run(&mut self, host: &str, port: u16) -> eyre::Result<()> {
        self.send_handshake(host, port).await?;
        self.send_login_start().await?;

        let mut tick_interval = tokio::time::interval(Duration::from_millis(50)); // 20 TPS
        let mut in_play = false;

        loop {
            tokio::select! {
                _ = tick_interval.tick(), if in_play => {
                    self.tick_physics();
                    self.send_position().await?;
                }
                result = self.read_packet() => {
                    let (packet_id, data) = result?;
                    if packet_id == -1 {
                        continue;
                    }

                    let mut cursor = Cursor::new(&data);

                    let should_continue = match self.state {
                        ConnectionState::Login => self.handle_login_packet(packet_id, &mut cursor).await?,
                        ConnectionState::Configuration => self.handle_configuration_packet(packet_id, &mut cursor).await?,
                        ConnectionState::Play => {
                            if !in_play {
                                in_play = true;
                                info!("Bot is now in play mode, starting to jump!");
                            }
                            self.handle_play_packet(packet_id, &mut cursor).await?
                        }
                        ConnectionState::Handshaking => true,
                    };

                    if !should_continue {
                        break;
                    }
                }
            }
        }

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

fn generate_bot_name() -> String {
    let mut rng = rand::thread_rng();
    let suffix: String = (0..6)
        .map(|_| {
            let c: u8 = rng.gen_range(0..36);
            if c < 10 {
                (b'0' + c) as char
            } else {
                (b'A' + c - 10) as char
            }
        })
        .collect();
    format!("Bot{}", suffix)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("mc_bot=info".parse()?),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    let host = args.get(1).map(|s| s.as_str()).unwrap_or("127.0.0.1");
    let port: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(25565);
    let player_name = args.get(3).cloned().unwrap_or_else(generate_bot_name);

    info!("MC Bot - Connecting to {}:{}", host, port);
    info!("Player: {}", player_name);

    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(&addr).await?;
    info!("Connected to {}", addr);

    let mut bot = Bot::new(stream, player_name);
    bot.run(host, port).await?;

    Ok(())
}
