//! Chunk components - chunk-related component definitions
//!
//! This module provides component definitions for chunks.
//! Systems that operate on these components are in `module-chunk`.

use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use flecs_ecs::prelude::*;
use module_loader::register_module;

// ============================================================================
// Chunk Components
// ============================================================================

/// Chunk coordinates
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

/// Pre-encoded chunk data for network transmission
#[derive(Component, Clone)]
pub struct ChunkData {
    pub encoded: Arc<Bytes>,
}

impl ChunkData {
    #[must_use]
    pub fn new(encoded: Bytes) -> Self {
        Self {
            encoded: Arc::new(encoded),
        }
    }
}

/// Tag: Chunk is fully loaded and ready
#[derive(Component, Default)]
pub struct ChunkLoaded;

/// Singleton: Spatial index for chunk lookup
#[derive(Component, Default)]
pub struct ChunkIndex {
    pub map: HashMap<ChunkPos, Entity>,
}

impl ChunkIndex {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, pos: ChunkPos, entity: Entity) {
        self.map.insert(pos, entity);
    }

    pub fn remove(&mut self, pos: &ChunkPos) -> Option<Entity> {
        self.map.remove(pos)
    }

    #[must_use]
    pub fn get(&self, pos: &ChunkPos) -> Option<Entity> {
        self.map.get(pos).copied()
    }
}

// ============================================================================
// Module
// ============================================================================

/// Chunk components module - registers chunk components
#[derive(Component)]
pub struct ChunkComponentsModule;

impl Module for ChunkComponentsModule {
    fn module(world: &World) {
        world.module::<ChunkComponentsModule>("chunk::components");

        // Register components
        world.component::<ChunkPos>();
        world.component::<ChunkData>();
        world.component::<ChunkLoaded>();

        // Set up ChunkIndex singleton
        world
            .component::<ChunkIndex>()
            .add_trait::<flecs::Singleton>();
        world.set(ChunkIndex::new());
    }
}

register_module! {
    name: "chunk-components",
    version: 1,
    module: ChunkComponentsModule,
    path: "::chunk::components",
}
