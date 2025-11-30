//! Chunk module - chunk loading and spatial indexing

mod world_gen;

use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use flecs_ecs::prelude::*;
use module_loader::register_module;

pub use world_gen::create_superflat_chunk;

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

/// Chunk module - handles chunk loading and indexing
#[derive(Component)]
pub struct ChunkModule;

impl Module for ChunkModule {
    fn module(world: &World) {
        world.module::<ChunkModule>("chunk");

        // Register components
        world.component::<ChunkPos>();
        world.component::<ChunkData>();
        world.component::<ChunkLoaded>();

        // Set up ChunkIndex singleton
        world
            .component::<ChunkIndex>()
            .add_trait::<flecs::Singleton>();
        world.set(ChunkIndex::new());

        // Observer: Add chunk to index when loaded
        world
            .observer_named::<flecs::OnSet, &ChunkPos>("ChunkIndexAdd")
            .with(ChunkLoaded)
            .each_entity(|e, pos| {
                e.world().get::<&mut ChunkIndex>(|index| {
                    index.insert(*pos, e.id());
                });
            });

        // Observer: Remove chunk from index when unloaded
        world
            .observer_named::<flecs::OnRemove, &ChunkPos>("ChunkIndexRemove")
            .each_entity(|e, pos| {
                e.world().get::<&mut ChunkIndex>(|index| {
                    index.remove(pos);
                });
            });
    }
}

/// Generate spawn chunks around origin
pub fn generate_spawn_chunks(world: &World, view_distance: i32) {
    for cx in -view_distance..=view_distance {
        for cz in -view_distance..=view_distance {
            let pos = ChunkPos::new(cx, cz);

            if let Ok(data) = create_superflat_chunk(cx, cz) {
                let name = format!("chunks::{}::{}", cx, cz);
                world
                    .entity_named(&name)
                    .set(pos)
                    .set(ChunkData::new(data))
                    .add(ChunkLoaded);
            }
        }
    }

    tracing::info!(
        "Generated {} spawn chunks",
        (view_distance * 2 + 1) * (view_distance * 2 + 1)
    );
}

register_module! {
    name: "chunk",
    version: 1,
    module: ChunkModule,
    path: "::chunk",
}
