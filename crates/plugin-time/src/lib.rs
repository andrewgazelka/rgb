//! Hot-reloadable time module plugin
//!
//! This plugin imports the time module from mc-server-lib.

use flecs_ecs::prelude::*;
use mc_server_lib::TimeModule;

/// Wrapper module for TimeModule
#[derive(Component)]
pub struct TimePluginModule;

impl Module for TimePluginModule {
    fn module(world: &World) {
        world.module::<TimePluginModule>("time_plugin");
        world.import::<TimeModule>();
    }
}

module_loader::register_module! {
    name: "time",
    version: 1,
    module: TimePluginModule,
    path: "::time_plugin",
}
