//! Component history tracking system using flecs-history
//!
//! This module sets up automatic history tracking for serializable components.

use flecs_ecs::prelude::*;
use flecs_history::prelude::*;

use crate::components::{
    ChunkPos, ChunkPosition, ConnectionId, EntityId, GameMode, Name, Position, ProtocolState,
    Rotation, ServerConfig, TpsTracker, Uuid, WorldTime,
};

/// Initialize history tracking for all serializable components.
///
/// This should be called after the world is created and before any entities are spawned.
/// Returns the HistoryTracker which should be stored and used for tick advancement.
pub fn init_history_tracking(world: &World) -> HistoryTracker {
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

    // Create history tracker
    let history = HistoryTracker::new(world);

    // Enable tracking for important components
    // These will record changes automatically via OnSet hooks
    history.track_component::<Position>(world);
    history.track_component::<Rotation>(world);
    history.track_component::<Name>(world);
    history.track_component::<Uuid>(world);
    history.track_component::<EntityId>(world);
    history.track_component::<GameMode>(world);

    history
}
