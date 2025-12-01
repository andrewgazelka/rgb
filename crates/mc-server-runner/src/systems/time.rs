//! Time systems

use rgb_ecs::{Entity, World};

use crate::components::{TpsTracker, WorldTime};

/// System: Tick world time forward
pub fn system_tick_world_time(world: &mut World) {
    if let Some(mut time) = world.get::<WorldTime>(Entity::WORLD) {
        time.tick();
        world.update(Entity::WORLD, time);
    }
}

/// System: Update TPS tracker
pub fn system_update_tps(world: &mut World, delta_time: f32) {
    if let Some(mut tps) = world.get::<TpsTracker>(Entity::WORLD) {
        tps.update(delta_time);
        world.update(Entity::WORLD, tps);
    }
}
