use flecs_ecs::prelude::*;

use crate::components::WorldTime;

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
    }
}
