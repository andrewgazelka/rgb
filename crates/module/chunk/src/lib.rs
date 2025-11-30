//! Chunk module - chunk loading and spatial indexing
//!
//! This is a stub module. The actual implementation lives in mc-server-lib::modules::chunk.
//! This module exists for the component/system separation architecture demonstration.

use flecs_ecs::prelude::*;
use module_loader::register_plugin;

/// Chunk module - handles chunk loading and indexing
#[derive(Component)]
pub struct ChunkModule;

impl Module for ChunkModule {
    fn module(world: &World) {
        world.module::<ChunkModule>("chunk");
        // Actual implementation in mc-server-lib
    }
}

register_plugin! {
    name: "chunk",
    version: 1,
    module: ChunkModule,
    path: "::chunk",
}
