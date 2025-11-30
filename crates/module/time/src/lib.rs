//! Time module - world time and TPS tracking
//!
//! This module provides:
//! - `WorldTime` - tracks world age and time of day
//! - `TpsTracker` - tracks ticks per second with EMAs
//! - Systems for ticking time forward

use flecs_ecs::prelude::*;

// ============================================================================
// Components
// ============================================================================

/// Singleton: World time tracking
#[derive(Component, Debug)]
pub struct WorldTime {
    pub world_age: i64,
    pub time_of_day: i64,
}

impl Default for WorldTime {
    fn default() -> Self {
        Self {
            world_age: 0,
            time_of_day: 6000, // Noon
        }
    }
}

impl WorldTime {
    /// Tick the world time forward
    pub fn tick(&mut self) {
        self.world_age += 1;
        self.time_of_day = (self.time_of_day + 1) % 24000;
    }
}

/// Singleton: TPS (ticks per second) tracking with exponential moving averages
#[derive(Component, Debug)]
pub struct TpsTracker {
    /// TPS with 5-second smoothing
    pub tps_5s: f32,
    /// TPS with 15-second smoothing
    pub tps_15s: f32,
    /// TPS with 1-minute smoothing
    pub tps_1m: f32,
}

impl Default for TpsTracker {
    fn default() -> Self {
        Self {
            tps_5s: 20.0,
            tps_15s: 20.0,
            tps_1m: 20.0,
        }
    }
}

impl TpsTracker {
    /// Update TPS values using exponential moving average
    pub fn update(&mut self, delta_time: f32) {
        if delta_time <= 0.0 {
            return;
        }

        let instant_tps = (1.0 / delta_time).min(1000.0);

        let alpha_5s = 1.0 - (-delta_time / 5.0_f32).exp();
        let alpha_15s = 1.0 - (-delta_time / 15.0_f32).exp();
        let alpha_1m = 1.0 - (-delta_time / 60.0_f32).exp();

        self.tps_5s += alpha_5s * (instant_tps - self.tps_5s);
        self.tps_15s += alpha_15s * (instant_tps - self.tps_15s);
        self.tps_1m += alpha_1m * (instant_tps - self.tps_1m);
    }
}

// ============================================================================
// Module
// ============================================================================

/// Time module - handles world time ticking and TPS tracking
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

module_loader::register_module! {
    name: "time",
    version: 1,
    module: TimeModule,
    path: "::time",
}
