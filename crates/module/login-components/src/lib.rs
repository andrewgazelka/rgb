//! Login components - player-related component definitions
//!
//! This module provides component definitions for players.
//! Systems that operate on these components are in `module-login`.

use std::sync::atomic::{AtomicI64, Ordering};

use flecs_ecs::prelude::*;
use module_loader::register_module;
use persist::PersistExt;
use serde::{Deserialize, Serialize};

// ============================================================================
// Player Components
// ============================================================================

/// Tag: Entity is a player
#[derive(Component, Default)]
#[flecs(meta)]
pub struct Player;

/// Player's username
#[derive(Component, Debug, Clone)]
#[flecs(meta)]
pub struct Name {
    pub value: String,
}

/// Player's UUID
#[derive(Component, Debug, Clone, Copy)]
#[flecs(meta)]
pub struct Uuid(pub u128);

impl From<&Uuid> for u128 {
    fn from(uuid: &Uuid) -> Self {
        uuid.0
    }
}

impl From<Uuid> for u128 {
    fn from(uuid: Uuid) -> Self {
        uuid.0
    }
}

/// Entity ID assigned by server (for protocol)
#[derive(Component, Debug, Clone, Copy)]
#[flecs(meta)]
pub struct EntityId {
    pub value: i32,
}

/// Player position in world
#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[flecs(meta)]
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

/// Player's current chunk position
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

/// Tag: Player needs initial spawn chunks sent
#[derive(Component, Default)]
#[flecs(meta)]
pub struct NeedsSpawnChunks;

/// Tag: Player has completed login and is in Play state
#[derive(Component, Default)]
#[flecs(meta)]
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
// Module
// ============================================================================

/// Login components module - registers player components
#[derive(Component)]
#[flecs(meta)]
pub struct LoginComponentsModule;

impl Module for LoginComponentsModule {
    fn module(world: &World) {
        world.module::<LoginComponentsModule>("login::components");

        // Register player components
        world.component::<Player>();
        world.component::<Name>();
        world.component::<Uuid>();
        world.component::<EntityId>();
        world.component::<Position>().persist::<Uuid>();
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
    }
}

register_module! {
    name: "login-components",
    version: 1,
    module: LoginComponentsModule,
    path: "::login::components",
}
