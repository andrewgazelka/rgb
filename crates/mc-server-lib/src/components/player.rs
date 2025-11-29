use flecs_ecs::prelude::*;

/// Tag: Entity is a player
#[derive(Component, Default)]
pub struct Player;

/// Player's username
#[derive(Component, Debug, Clone)]
#[flecs(meta)]
pub struct Name {
    pub value: String,
}

/// Player's UUID (stored as u128, serialized as hex string for flecs explorer)
#[derive(Component, Debug, Clone, Copy)]
pub struct Uuid(pub u128);

/// Register Uuid with opaque serialization (u128 -> hex string)
pub fn register_uuid_meta(world: &World) {
    world
        .component::<Uuid>()
        .opaque_id(world.component_untyped().member(String::id(), "uuid"))
        .serialize(|s: &Serializer, data: &Uuid| {
            s.member("uuid");
            let hex = format!("{:032x}", data.0);
            s.value(&hex);
            0
        });
}

/// Entity ID assigned by server (for protocol)
#[derive(Component, Debug, Clone, Copy)]
#[flecs(meta)]
pub struct EntityId {
    pub value: i32,
}

/// Player position in world
#[derive(Component, Debug, Clone, Copy, Default)]
#[flecs(meta)]
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
#[flecs(meta)]
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
#[flecs(meta)]
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
#[flecs(meta)]
pub struct GameMode {
    pub value: u8,
}

impl GameMode {
    pub const SURVIVAL: Self = Self { value: 0 };
    pub const CREATIVE: Self = Self { value: 1 };
    pub const ADVENTURE: Self = Self { value: 2 };
    pub const SPECTATOR: Self = Self { value: 3 };
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
#[flecs(meta)]
pub struct AwaitingTeleportAck {
    pub teleport_id: i32,
}
