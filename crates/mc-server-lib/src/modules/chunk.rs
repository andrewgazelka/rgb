use flecs_ecs::prelude::*;

use crate::components::{ChunkData, ChunkIndex, ChunkLoaded, ChunkPos};
use crate::world_gen::create_superflat_chunk;

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

        // ChunkIndex singleton trait is registered in create_world()

        // Observer: Add chunk to index when loaded
        world
            .observer::<flecs::OnSet, &ChunkPos>()
            .with(ChunkLoaded)
            .each_entity(|e, pos| {
                e.world().get::<&mut ChunkIndex>(|index| {
                    index.insert(*pos, e.id());
                });
            });

        // Observer: Remove chunk from index when unloaded
        world
            .observer::<flecs::OnRemove, &ChunkPos>()
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

            // Generate chunk data
            if let Ok(data) = create_superflat_chunk(cx, cz) {
                world
                    .entity()
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
