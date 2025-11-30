//! Login module - handles login flow
//!
//! This is a stub module. The actual implementation lives in mc-server-lib::modules::login.
//! This module exists for the component/system separation architecture demonstration.

use flecs_ecs::prelude::*;
use module_loader::register_plugin;

/// Login module - handles login flow
#[derive(Component)]
pub struct LoginModule;

impl Module for LoginModule {
    fn module(world: &World) {
        world.module::<LoginModule>("login");
        // Actual implementation in mc-server-lib
    }
}

register_plugin! {
    name: "login",
    version: 1,
    module: LoginModule,
    path: "::login",
}
