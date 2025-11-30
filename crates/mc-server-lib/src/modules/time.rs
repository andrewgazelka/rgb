use flecs_ecs::prelude::*;

use crate::components::{TpsTracker, WorldTime};

/// Time module - handles world time ticking
#[derive(Component)]
pub struct TimeModule;

impl Module for TimeModule {
    fn module(world: &World) {
        world.module::<TimeModule>("time");

        // Register and set up singletons
        world.component::<WorldTime>();
        world.component::<TpsTracker>();
        world
            .component::<WorldTime>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<TpsTracker>()
            .add_trait::<flecs::Singleton>();
        world.set(WorldTime::default());
        world.set(TpsTracker::default());

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
