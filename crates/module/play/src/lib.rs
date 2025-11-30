//! Play module - handles play state
//!
//! This is a stub module. The actual implementation lives in mc-server-lib::modules::play.
//! This module exists for the component/system separation architecture demonstration.

use flecs_ecs::prelude::*;
use module_loader::register_plugin;

/// Play module - handles play state
#[derive(Component)]
pub struct PlayModule;

impl Module for PlayModule {
    fn module(world: &World) {
        world.module::<PlayModule>("play");
        // Actual implementation in mc-server-lib
    }
}

register_plugin! {
    name: "play",
    version: 1,
    module: PlayModule,
    path: "::play",
}
