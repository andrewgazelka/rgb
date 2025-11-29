use std::collections::HashMap;
use std::sync::atomic::AtomicI64;

use flecs_ecs::prelude::*;

use super::ChunkPos;

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
