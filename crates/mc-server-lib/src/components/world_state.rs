use std::collections::HashMap;
use std::sync::atomic::AtomicI64;

use flecs_ecs::prelude::*;

use super::ChunkPos;

/// Singleton: World time tracking
#[derive(Component, Debug)]
#[flecs(meta)]
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

/// Singleton: Entity ID counter for protocol
#[derive(Component)]
pub struct EntityIdCounter(pub AtomicI64);

impl Default for EntityIdCounter {
    fn default() -> Self {
        Self(AtomicI64::new(1))
    }
}

impl EntityIdCounter {
    /// Get the next entity ID
    pub fn next(&self) -> i32 {
        use std::sync::atomic::Ordering;
        self.0.fetch_add(1, Ordering::Relaxed) as i32
    }
}

/// Singleton: Spatial index for chunk lookup
#[derive(Component, Default)]
pub struct ChunkIndex {
    pub map: HashMap<ChunkPos, flecs_ecs::core::Entity>,
}

impl ChunkIndex {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, pos: ChunkPos, entity: flecs_ecs::core::Entity) {
        self.map.insert(pos, entity);
    }

    pub fn remove(&mut self, pos: &ChunkPos) -> Option<flecs_ecs::core::Entity> {
        self.map.remove(pos)
    }

    #[must_use]
    pub fn get(&self, pos: &ChunkPos) -> Option<flecs_ecs::core::Entity> {
        self.map.get(pos).copied()
    }
}

/// Singleton: Connection ID counter
#[derive(Component)]
pub struct ConnectionIdCounter(pub AtomicI64);

impl Default for ConnectionIdCounter {
    fn default() -> Self {
        Self(AtomicI64::new(1))
    }
}

impl ConnectionIdCounter {
    /// Get the next connection ID
    pub fn next(&self) -> u64 {
        use std::sync::atomic::Ordering;
        self.0.fetch_add(1, Ordering::Relaxed) as u64
    }
}

/// Singleton: TPS (ticks per second) tracking with exponential moving averages
#[derive(Component, Debug)]
pub struct TpsTracker {
    /// TPS with 5-second smoothing (alpha ~= 1 - e^(-dt/5))
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
    /// alpha = 1 - e^(-dt/tau) where tau is the time constant
    pub fn update(&mut self, delta_time: f32) {
        if delta_time <= 0.0 {
            return;
        }

        let instant_tps = (1.0 / delta_time).min(1000.0); // Cap at 1000 TPS

        // Exponential moving average: new = old + alpha * (sample - old)
        // alpha = 1 - e^(-dt/tau)
        let alpha_5s = 1.0 - (-delta_time / 5.0_f32).exp();
        let alpha_15s = 1.0 - (-delta_time / 15.0_f32).exp();
        let alpha_1m = 1.0 - (-delta_time / 60.0_f32).exp();

        self.tps_5s += alpha_5s * (instant_tps - self.tps_5s);
        self.tps_15s += alpha_15s * (instant_tps - self.tps_15s);
        self.tps_1m += alpha_1m * (instant_tps - self.tps_1m);
    }
}
