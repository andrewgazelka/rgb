//! Skript runtime module for Minecraft servers.
//!
//! This module provides execution of Skript scripts, integrating with Flecs ECS.
//!
//! # Architecture
//!
//! - `skript-lang`: Parser and AST (language frontend)
//! - `module-skript`: Runtime, execution, Minecraft integration (this crate)
//!
//! See `plan/overview.md` for the implementation roadmap.

mod value;

pub use value::Value;

use flecs_ecs::prelude::*;

/// Skript module for Flecs.
#[derive(Component)]
pub struct Skript;

impl Module for Skript {
    fn module(world: &World) {
        world.module::<Skript>("skript");

        // TODO: Register components and systems
        tracing::info!("Skript module loaded");
    }
}
