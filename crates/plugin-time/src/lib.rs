//! Hot-reloadable time module plugin
//!
//! This plugin imports the time module from mc-server-lib.

use flecs_ecs::prelude::*;
use mc_server_lib::TimeModule;

/// Plugin version - change this to verify hot-reload works
pub const PLUGIN_VERSION: u32 = 1;

/// Load the module into the world
#[unsafe(no_mangle)]
pub fn plugin_load(world: &World) {
    world.import::<TimeModule>();
}

/// Unload the module from the world
#[unsafe(no_mangle)]
pub fn plugin_unload(world: &World) {
    if let Some(module_entity) = world.try_lookup("::time") {
        module_entity.destruct();
    }
}

/// Get the plugin name
#[unsafe(no_mangle)]
pub fn plugin_name() -> &'static str {
    "time"
}

/// Get the plugin version
#[unsafe(no_mangle)]
pub fn plugin_version() -> u32 {
    PLUGIN_VERSION
}
