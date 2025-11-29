use flecs_ecs::prelude::*;

use crate::components::{TpsTracker, WorldTime};

/// Time module - handles world time ticking
#[derive(Component)]
pub struct TimeModule;

impl Module for TimeModule {
    fn module(world: &World) {
        world.module::<TimeModule>("time");

        // WorldTime singleton trait is registered in create_world() before modules are imported

        // Tick world time each frame
        world
            .system_named::<&mut WorldTime>("TickWorldTime")
            .each(|time| {
                time.tick();
            });

        // Update TPS tracker each frame using delta_time
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
