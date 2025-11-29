use std::sync::Arc;

use bytes::Bytes;
use flecs_ecs::prelude::*;

/// Chunk coordinates
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[flecs(meta)]
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
    /// Full packet data (without packet ID prefix)
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

/// Tag: Chunk data needs regeneration
#[derive(Component, Default)]
pub struct ChunkDirty;
