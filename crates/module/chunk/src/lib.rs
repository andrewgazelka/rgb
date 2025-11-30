//! Chunk module - chunk loading and spatial indexing

mod world_gen;

use flecs_ecs::prelude::*;
use module_loader::register_module;

pub use world_gen::create_superflat_chunk;

// Re-export components for convenience
pub use module_chunk_components::{
    ChunkComponentsModule, ChunkData, ChunkIndex, ChunkLoaded, ChunkPos,
};

// ============================================================================
// Module
// ============================================================================

/// Chunk module - handles chunk loading and indexing
#[derive(Component)]
pub struct ChunkModule;

impl Module for ChunkModule {
    fn module(world: &World) {
        world.module::<ChunkModule>("chunk");

        // Import component module
        world.import::<ChunkComponentsModule>();

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
