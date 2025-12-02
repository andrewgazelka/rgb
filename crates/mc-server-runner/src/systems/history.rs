//! Component serialization setup
//!
//! This module registers components as serializable for persistence.

use flecs_ecs::prelude::*;
use flecs_history::prelude::*;

use crate::components::{
    ChunkPos, ChunkPosition, ConnectionId, EntityId, GameMode, Name, Position, ProtocolState,
    Rotation, ServerConfig, TpsTracker, Uuid, WorldTime,
};

/// Initialize serialization for all persistable components.
///
/// This should be called after the world is created and before any entities are spawned.
pub fn init_serialization(world: &World) {
    // Register components as serializable
    world.component::<Position>().serializable::<Position>();
    world.component::<Rotation>().serializable::<Rotation>();
    world.component::<Name>().serializable::<Name>();
    world.component::<Uuid>().serializable::<Uuid>();
    world.component::<EntityId>().serializable::<EntityId>();
    world.component::<GameMode>().serializable::<GameMode>();
    world
        .component::<ChunkPosition>()
        .serializable::<ChunkPosition>();
    world.component::<ChunkPos>().serializable::<ChunkPos>();
    world
        .component::<ConnectionId>()
        .serializable::<ConnectionId>();
    world
        .component::<ProtocolState>()
        .serializable::<ProtocolState>();
    world.component::<WorldTime>().serializable::<WorldTime>();
    world.component::<TpsTracker>().serializable::<TpsTracker>();
    world
        .component::<ServerConfig>()
        .serializable::<ServerConfig>();
}
