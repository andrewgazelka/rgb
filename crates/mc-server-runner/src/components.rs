//! All ECS components for the Minecraft server

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use flecs_ecs::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// Network Components
// ============================================================================

/// Packet received from async network layer
#[derive(Debug)]
pub struct IncomingPacket {
    pub connection_id: u64,
    pub packet_id: i32,
    pub data: Bytes,
}

/// Event signaling a connection has been closed
#[derive(Debug)]
pub struct DisconnectEvent {
    pub connection_id: u64,
}

/// Packet to send via async network layer
#[derive(Debug)]
pub struct OutgoingPacket {
    pub connection_id: u64,
    pub data: Bytes,
}

/// Global: Receiver for incoming packets from async layer
#[derive(Component)]
pub struct NetworkIngress {
    pub rx: Receiver<IncomingPacket>,
}

/// Global: Sender for outgoing packets to async layer
#[derive(Component)]
pub struct NetworkEgress {
    pub tx: Sender<OutgoingPacket>,
}

/// Global: Receiver for disconnect events
#[derive(Component)]
pub struct DisconnectIngress {
    pub rx: Receiver<DisconnectEvent>,
}

/// Tag: Entity is a network connection
#[derive(Component, Default)]
pub struct Connection;

/// Unique ID for routing packets to correct connection
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub u64);

impl ConnectionId {
    /// Generate the byte key for named entity lookup.
    ///
    /// Format: `b"conn" ++ id.to_le_bytes()` (12 bytes)
    #[must_use]
    pub fn to_key(self) -> [u8; 12] {
        let mut key = [0u8; 12];
        key[0..4].copy_from_slice(b"conn");
        key[4..12].copy_from_slice(&self.0.to_le_bytes());
        key
    }
}

/// Current protocol state of the connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ConnectionState {
    #[default]
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ProtocolState(pub ConnectionState);

/// Buffer for incoming/outgoing packets per connection
#[derive(Component, Default)]
pub struct PacketBuffer {
    pub incoming: VecDeque<(i32, Bytes)>,
    pub outgoing: VecDeque<Bytes>,
}

impl PacketBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_incoming(&mut self, packet_id: i32, data: Bytes) {
        self.incoming.push_back((packet_id, data));
    }

    pub fn pop_incoming(&mut self) -> Option<(i32, Bytes)> {
        self.incoming.pop_front()
    }

    pub fn push_outgoing(&mut self, data: Bytes) {
        self.outgoing.push_back(data);
    }

    pub fn pop_outgoing(&mut self) -> Option<Bytes> {
        self.outgoing.pop_front()
    }
}

/// Global: Temporary buffer for packets arriving before connection entity is ready.
///
/// Note: Connection ID -> Entity mapping is done via named entities (world.lookup).
#[derive(Component, Default)]
pub struct PendingPackets {
    /// Packets for newly created connections (deferred until next tick)
    pub packets: Vec<(u64, i32, Bytes)>,
}

/// Global: Index for looking up connection entities by ID
#[derive(Component, Default)]
pub struct ConnectionIndex {
    pub map: hashbrown::HashMap<u64, Entity>,
}

// ============================================================================
// Player Components
// ============================================================================

/// Tag: Entity is a player
#[derive(Component, Default)]
pub struct Player;

/// Player's username
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    pub value: String,
}

/// Player's UUID
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Uuid(pub u128);

/// Entity ID assigned by server (for protocol)
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntityId {
    pub value: i32,
}

/// Player position in world
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    /// Default spawn position for new players
    pub const SPAWN: Self = Self {
        x: 0.0,
        y: 100.0,
        z: 0.0,
    };

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
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
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
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
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
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
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

/// Global: Entity ID counter for protocol
#[derive(Component)]
pub struct EntityIdCounter(pub Arc<AtomicI64>);

impl Default for EntityIdCounter {
    fn default() -> Self {
        Self(Arc::new(AtomicI64::new(1)))
    }
}

impl EntityIdCounter {
    pub fn next(&self) -> i32 {
        self.0.fetch_add(1, Ordering::Relaxed) as i32
    }
}

// ============================================================================
// Chunk Components
// ============================================================================

/// Chunk coordinates
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Generate the byte key for named entity lookup.
    ///
    /// Format: `b"chunk" ++ x.to_le_bytes() ++ z.to_le_bytes()` (13 bytes)
    #[must_use]
    pub fn to_key(self) -> [u8; 13] {
        let mut key = [0u8; 13];
        key[0..5].copy_from_slice(b"chunk");
        key[5..9].copy_from_slice(&self.x.to_le_bytes());
        key[9..13].copy_from_slice(&self.z.to_le_bytes());
        key
    }
}

/// Pre-encoded chunk data for network transmission
#[derive(Component)]
pub struct ChunkData {
    pub encoded: Bytes,
}

impl ChunkData {
    #[must_use]
    pub fn new(encoded: Bytes) -> Self {
        Self { encoded }
    }
}

/// Tag: Chunk is fully loaded and ready
#[derive(Component, Default)]
pub struct ChunkLoaded;

// ============================================================================
// Server Config (Global)
// ============================================================================

/// Global: Server configuration
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Maximum number of players allowed
    pub max_players: i32,
    /// Server description shown in server list
    pub motd: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_players: 20_000,
            motd: "A Rust Minecraft Server (Flecs ECS)".to_string(),
        }
    }
}

// ============================================================================
// Time Components (Global)
// ============================================================================

/// Global: World time tracking
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorldTime {
    pub world_age: i64,
    pub time_of_day: i64,
}

impl Default for WorldTime {
    fn default() -> Self {
        Self {
            world_age: 0,
            time_of_day: 6000, // Noon
        }
    }
}

impl WorldTime {
    /// Tick the world time forward
    pub fn tick(&mut self) {
        self.world_age += 1;
        self.time_of_day = (self.time_of_day + 1) % 24000;
    }
}

/// Global: TPS (ticks per second) tracking with exponential moving averages
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TpsTracker {
    /// TPS with 5-second smoothing
    pub tps_5s: f32,
    /// TPS with 15-second smoothing
    pub tps_15s: f32,
    /// TPS with 1-minute smoothing
    pub tps_1m: f32,
}

impl Default for TpsTracker {
    fn default() -> Self {
        Self {
            tps_5s: 20.0,
            tps_15s: 20.0,
            tps_1m: 20.0,
        }
    }
}

impl TpsTracker {
    /// Update TPS values using exponential moving average
    pub fn update(&mut self, delta_time: f32) {
        if delta_time <= 0.0 {
            return;
        }

        let instant_tps = (1.0 / delta_time).min(1000.0);

        let alpha_5s = 1.0 - (-delta_time / 5.0_f32).exp();
        let alpha_15s = 1.0 - (-delta_time / 15.0_f32).exp();
        let alpha_1m = 1.0 - (-delta_time / 60.0_f32).exp();

        self.tps_5s += alpha_5s * (instant_tps - self.tps_5s);
        self.tps_15s += alpha_15s * (instant_tps - self.tps_15s);
        self.tps_1m += alpha_1m * (instant_tps - self.tps_1m);
    }
}

/// Global: Delta time for current tick
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DeltaTime(pub f32);
