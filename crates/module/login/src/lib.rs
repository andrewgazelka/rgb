//! Login module - handles login flow

use std::sync::atomic::{AtomicI64, Ordering};

use bytes::{BufMut, Bytes, BytesMut};
use flecs_ecs::prelude::*;
use mc_protocol::{write_varint, Decode, Encode};
use module_network_components::{Connection, ConnectionState, PacketBuffer, ProtocolState};
use module_loader::register_module;
use tracing::{debug, info};

// ============================================================================
// Player Components (defined here, used by other modules)
// ============================================================================

/// Tag: Entity is a player
#[derive(Component, Default)]
pub struct Player;

/// Player's username
#[derive(Component, Debug, Clone)]
pub struct Name {
    pub value: String,
}

/// Player's UUID
#[derive(Component, Debug, Clone, Copy)]
pub struct Uuid(pub u128);

/// Entity ID assigned by server (for protocol)
#[derive(Component, Debug, Clone, Copy)]
pub struct EntityId {
    pub value: i32,
}

/// Player position in world
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn chunk_pos(&self) -> (i32, i32) {
        ((self.x as i32) >> 4, (self.z as i32) >> 4)
    }
}

/// Player rotation
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl Rotation {
    #[must_use]
    pub const fn new(yaw: f32, pitch: f32) -> Self {
        Self { yaw, pitch }
    }
}

/// Player's current chunk position
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl ChunkPosition {
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

/// Player game mode
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GameMode {
    pub value: u8,
}

impl GameMode {
    pub const SURVIVAL: Self = Self { value: 0 };
    pub const CREATIVE: Self = Self { value: 1 };
    pub const ADVENTURE: Self = Self { value: 2 };
    pub const SPECTATOR: Self = Self { value: 3 };
}

/// Tag: Player needs initial spawn chunks sent
#[derive(Component, Default)]
pub struct NeedsSpawnChunks;

/// Tag: Player has completed login and is in Play state
#[derive(Component, Default)]
pub struct InPlayState;

/// Singleton: Entity ID counter for protocol
#[derive(Component)]
pub struct EntityIdCounter(pub AtomicI64);

impl Default for EntityIdCounter {
    fn default() -> Self {
        Self(AtomicI64::new(1))
    }
}

impl EntityIdCounter {
    pub fn next(&self) -> i32 {
        self.0.fetch_add(1, Ordering::Relaxed) as i32
    }
}

// ============================================================================
// Packet helpers
// ============================================================================

fn encode_packet(packet_id: i32, data: &[u8]) -> Bytes {
    let mut packet_id_bytes = Vec::new();
    write_varint(&mut packet_id_bytes, packet_id).expect("varint write");

    let length = packet_id_bytes.len() + data.len();
    let mut length_bytes = Vec::new();
    write_varint(&mut length_bytes, length as i32).expect("varint write");

    let mut buf = BytesMut::with_capacity(length_bytes.len() + packet_id_bytes.len() + data.len());
    buf.put_slice(&length_bytes);
    buf.put_slice(&packet_id_bytes);
    buf.put_slice(data);
    buf.freeze()
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

fn parse_login_start(data: &[u8]) -> eyre::Result<(String, u128)> {
    let mut cursor = std::io::Cursor::new(data);
    let name = String::decode(&mut cursor)?;
    let uuid = mc_protocol::Uuid::decode(&mut cursor)?;
    Ok((name, uuid.0))
}

fn create_login_success(uuid: u128, name: &str) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    mc_protocol::Uuid(uuid).encode(&mut data)?;
    name.to_string().encode(&mut data)?;
    write_varint(&mut data, 0)?; // 0 properties
    Ok(data)
}

fn create_known_packs() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, 1)?;
    "minecraft".to_string().encode(&mut data)?;
    "core".to_string().encode(&mut data)?;
    "1.21".to_string().encode(&mut data)?;
    Ok(data)
}

fn try_parse_login(data: &[u8]) -> Option<(String, u128)> {
    parse_login_start(data).ok()
}

fn send_login_success(buffer: &mut PacketBuffer, uuid: u128, name: &str) {
    if let Ok(response_data) = create_login_success(uuid, name) {
        let packet = encode_packet(2, &response_data);
        buffer.push_outgoing(packet);
    }
}

fn send_known_packs(buffer: &mut PacketBuffer) {
    if let Ok(data) = create_known_packs() {
        let packet = encode_packet(14, &data);
        buffer.push_outgoing(packet);
    }
}

// ============================================================================
// Module
// ============================================================================

/// Login module - handles login flow
#[derive(Component)]
pub struct LoginModule;

impl Module for LoginModule {
    fn module(world: &World) {
        world.module::<LoginModule>("login");

        // Register player components
        world.component::<Player>();
        world.component::<Name>();
        world.component::<Uuid>();
        world.component::<EntityId>();
        world.component::<Position>();
        world.component::<Rotation>();
        world.component::<ChunkPosition>();
        world.component::<GameMode>();
        world.component::<NeedsSpawnChunks>();
        world.component::<InPlayState>();

        // Set up EntityIdCounter singleton
        world
            .component::<EntityIdCounter>()
            .add_trait::<flecs::Singleton>();
        world.set(EntityIdCounter::default());

        // Handle login packets
        world
            .system_named::<(&mut ProtocolState, &mut PacketBuffer, &EntityIdCounter)>(
                "HandleLogin",
            )
            .with(Connection)
            .each_entity(|e, (state, buffer, entity_counter)| {
                if state.0 != ConnectionState::Login {
                    return;
                }

                while let Some((packet_id, data)) = buffer.pop_incoming() {
                    debug!("HandleLogin: got packet_id={}", packet_id);
                    match packet_id {
                        0 => {
                            // Login Start
                            if let Some((name, _uuid)) = try_parse_login(&data) {
                                let player_uuid = offline_uuid(&name);
                                info!("Login from: {} (uuid: {:032x})", &name, player_uuid);

                                let player_path = format!("players::{}", name);
                                e.set_name(&player_path);

                                let entity_id = entity_counter.next();
                                e.add(Player);
                                e.set(Name { value: name.clone() });
                                e.set(Uuid(player_uuid));
                                e.set(EntityId { value: entity_id });
                                e.set(Position::new(0.0, 4.0, 0.0));
                                e.set(Rotation::new(0.0, 0.0));
                                e.set(ChunkPosition::new(0, 0));
                                e.set(GameMode::CREATIVE);

                                send_login_success(buffer, player_uuid, &name);
                                info!("Sent Login Success, waiting for Login Acknowledged");
                            }
                        }
                        3 => {
                            // Login Acknowledged
                            info!("Login Acknowledged, transitioning to Configuration");
                            state.0 = ConnectionState::Configuration;
                            send_known_packs(buffer);
                            debug!("Sent Known Packs");
                        }
                        _ => {
                            debug!("Unknown login packet: {}", packet_id);
                        }
                    }
                }
            });
    }
}

register_module! {
    name: "login",
    version: 1,
    module: LoginModule,
    path: "::login",
}
