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

/// Plugin version - change this to verify hot-reload works
pub const PLUGIN_VERSION: u32 = 1;

/// Server module - imports all sub-modules
#[derive(Component)]
pub struct ServerModule;

impl Module for ServerModule {
    fn module(world: &World) {
        world.module::<ServerModule>("server");

        // Import all server modules
        // Note: TimeModule must be imported before PlayModule since PlayModule
        // creates systems that query WorldTime/TpsTracker singletons
        world.import::<NetworkModule>();
        world.import::<PacketDispatchModule>();
        world.import::<TimeModule>(); // Must be before PlayModule (sets up WorldTime singleton)
        world.import::<ChunkModule>(); // Must be before PlayModule (sets up ChunkIndex singleton)
        world.import::<HandshakeModule>();
        world.import::<LoginModule>();
        world.import::<ConfigurationModule>();
        world.import::<PlayModule>();
    }
}

/// Load the module into the world
#[unsafe(no_mangle)]
pub fn plugin_load(world: &World) {
    world.import::<ServerModule>();
}

/// Unload the module from the world
#[unsafe(no_mangle)]
pub fn plugin_unload(world: &World) {
    if let Some(module_entity) = world.try_lookup("::server") {
        module_entity.destruct();
    }
}

/// Get the plugin name
#[unsafe(no_mangle)]
pub fn plugin_name() -> &'static str {
    "server"
}

/// Get the plugin version
#[unsafe(no_mangle)]
pub fn plugin_version() -> u32 {
    PLUGIN_VERSION
}
