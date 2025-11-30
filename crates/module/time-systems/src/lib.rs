//! Time systems module - systems for ticking world time and tracking TPS
//!
//! This module provides systems that operate on time components.
//! Depends on `module-time-components` for component definitions.

use flecs_ecs::prelude::*;
use module_loader::register_plugin;
use module_time_components::{TimeComponentsModule, TpsTracker, WorldTime};

// ============================================================================
// Module
// ============================================================================

/// Time systems module - handles world time ticking and TPS tracking
#[derive(Component)]
pub struct TimeSystemsModule;

impl Module for TimeSystemsModule {
    fn module(world: &World) {
        world.module::<TimeSystemsModule>("time::systems");

        // Import component module (ensures components exist)
        world.import::<TimeComponentsModule>();

        // Tick world time each frame
        world
            .system_named::<&mut WorldTime>("TickWorldTime")
            .each(|time| {
                time.tick();
            });

        // Update TPS tracker each frame
        world
            .system_named::<&mut TpsTracker>("UpdateTpsTracker")
            .run(|mut it| {
                while it.next() {
                    let delta_time = it.delta_time();
                    let mut tps = it.field_mut::<TpsTracker>(0);
                    for i in it.iter() {
                        tps[i].update(delta_time);
                    }
                }
            });
    }
}

// ============================================================================
// Plugin exports
// ============================================================================

register_module! {
    name: "time-systems",
    version: 1,
    module: TimeSystemsModule,
    path: "::time::systems",
}
