//! Hot-reloadable Minecraft server plugin
//!
//! This plugin contains all the core server functionality:
//! - Network handling (handshake, login, configuration, play)
//! - Chunk generation and management
//! - World time and TPS tracking
//! - Packet dispatch

use flecs_ecs::prelude::*;
use mc_server_lib::{
    ChunkModule, ConfigurationModule, HandshakeModule, LoginModule, NetworkModule,
    PacketDispatchModule, PlayModule, TimeModule,
};

/// Server module - imports all sub-modules
#[derive(Component)]
#[flecs(meta)]
pub struct ServerModule;

impl Module for ServerModule {
    fn module(world: &World) {
        world.module::<ServerModule>("server");

        // Import all server modules
        // Order matters! Modules that set up singletons must come before modules that query them.
        world.import::<NetworkModule>(); // Sets up ConnectionIndex
        world.import::<PacketDispatchModule>();
        world.import::<TimeModule>(); // Sets up WorldTime, TpsTracker
        world.import::<ChunkModule>(); // Sets up ChunkIndex
        world.import::<LoginModule>(); // Sets up EntityIdCounter
        world.import::<HandshakeModule>();
        world.import::<ConfigurationModule>();
        world.import::<PlayModule>(); // Queries WorldTime, TpsTracker, ChunkIndex, EntityId
    }
}

module_loader::register_module! {
    name: "server",
    version: 1,
    module: ServerModule,
    path: "::server",
}
