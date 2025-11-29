use flecs_ecs::prelude::*;

/// Tag: Entity is a player
#[derive(Component, Default)]
pub struct Player;

/// Player's username
#[derive(Component, Debug, Clone)]
pub struct PlayerName(pub String);

/// Player's UUID
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerUuid(pub u128);

/// Entity ID assigned by server (for protocol)
#[derive(Component, Debug, Clone, Copy)]
pub struct EntityId(pub i32);

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

    /// Get chunk coordinates for this position
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

/// Player's current chunk position (for view distance tracking)
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
pub struct GameMode(pub u8);

impl GameMode {
    pub const SURVIVAL: Self = Self(0);
    pub const CREATIVE: Self = Self(1);
    pub const ADVENTURE: Self = Self(2);
    pub const SPECTATOR: Self = Self(3);
}

/// Tag: Player has completed login and is in Play state
#[derive(Component, Default)]
pub struct InPlayState;

/// Tag: Player needs initial spawn chunks sent
#[derive(Component, Default)]
pub struct NeedsSpawnChunks;

/// Tag: Player's view center needs update (chunk position changed)
#[derive(Component, Default)]
pub struct ViewCenterDirty;

/// Tag: Player is awaiting teleport confirmation
#[derive(Component, Debug, Clone, Copy)]
pub struct AwaitingTeleportAck(pub i32);
