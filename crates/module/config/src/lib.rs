//! Configuration module - handles configuration phase
//!
//! This is a stub module. The actual implementation lives in mc-server-lib::modules::config.
//! This module exists for the component/system separation architecture demonstration.

use flecs_ecs::prelude::*;
use module_loader::register_plugin;

/// Configuration module - handles configuration phase
#[derive(Component)]
pub struct ConfigurationModule;

impl Module for ConfigurationModule {
    fn module(world: &World) {
        world.module::<ConfigurationModule>("configuration");
        // Actual implementation in mc-server-lib
    }
}

register_plugin! {
    name: "configuration",
    version: 1,
    module: ConfigurationModule,
    path: "::configuration",
}
