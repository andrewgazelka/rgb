use flecs_ecs::prelude::*;

use crate::components::WorldTime;

/// Time module - handles world time ticking
#[derive(Component)]
pub struct TimeModule;

impl Module for TimeModule {
    fn module(world: &World) {
        world.module::<TimeModule>("time");

        // Register WorldTime singleton
        world
            .component::<WorldTime>()
            .add_trait::<flecs::Singleton>();

        // Tick world time each frame
        world
            .system_named::<&mut WorldTime>("TickWorldTime")
            .each(|time| {
                time.tick();
            });
    }
}
