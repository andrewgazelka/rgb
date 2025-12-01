//! Singleton components - global state that exists once per world.
//!
//! Unlike regular components which are attached to entities,
//! singletons exist independently and are accessed by type.
//!
//! Examples: GameConfig, Time, InputState, WorldBounds

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Marker trait for singleton components.
///
/// Singletons are global components that exist once per world,
/// not attached to any entity.
pub trait Singleton: Send + Sync + 'static {}

// Blanket implementation
impl<T: Send + Sync + 'static> Singleton for T {}

/// Type-erased storage for a singleton value.
struct SingletonEntry {
    /// The singleton value.
    value: Box<dyn Any + Send + Sync>,
}

impl SingletonEntry {
    fn new<T: Singleton>(value: T) -> Self {
        Self {
            value: Box::new(value),
        }
    }

    fn get<T: 'static>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }

    fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.value.downcast_mut()
    }
}

/// Storage for singleton components.
///
/// Provides typed access to global state that exists once per world.
#[derive(Default)]
pub struct SingletonStorage {
    /// Map from TypeId to singleton value.
    singletons: HashMap<TypeId, SingletonEntry>,
}

impl SingletonStorage {
    /// Create new empty singleton storage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a singleton, replacing any existing value of the same type.
    pub fn insert<T: Singleton>(&mut self, value: T) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let old = self.singletons.insert(type_id, SingletonEntry::new(value));
        old.and_then(|entry| entry.value.downcast().ok().map(|b| *b))
    }

    /// Get a reference to a singleton.
    #[must_use]
    pub fn get<T: Singleton>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.singletons.get(&type_id).and_then(SingletonEntry::get)
    }

    /// Get a mutable reference to a singleton.
    #[must_use]
    pub fn get_mut<T: Singleton>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.singletons
            .get_mut(&type_id)
            .and_then(SingletonEntry::get_mut)
    }

    /// Get a singleton, inserting a default value if it doesn't exist.
    pub fn get_or_insert<T: Singleton + Default>(&mut self) -> &mut T {
        let type_id = TypeId::of::<T>();
        self.singletons
            .entry(type_id)
            .or_insert_with(|| SingletonEntry::new(T::default()))
            .get_mut()
            .expect("Type mismatch in singleton storage")
    }

    /// Get a singleton, inserting a value from a closure if it doesn't exist.
    pub fn get_or_insert_with<T: Singleton, F: FnOnce() -> T>(&mut self, f: F) -> &mut T {
        let type_id = TypeId::of::<T>();
        self.singletons
            .entry(type_id)
            .or_insert_with(|| SingletonEntry::new(f()))
            .get_mut()
            .expect("Type mismatch in singleton storage")
    }

    /// Remove a singleton, returning it if it existed.
    pub fn remove<T: Singleton>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.singletons
            .remove(&type_id)
            .and_then(|entry| entry.value.downcast().ok().map(|b| *b))
    }

    /// Check if a singleton exists.
    #[must_use]
    pub fn contains<T: Singleton>(&self) -> bool {
        self.singletons.contains_key(&TypeId::of::<T>())
    }

    /// Get the number of singletons.
    #[must_use]
    pub fn len(&self) -> usize {
        self.singletons.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.singletons.is_empty()
    }

    /// Clear all singletons.
    pub fn clear(&mut self) {
        self.singletons.clear();
    }
}

impl std::fmt::Debug for SingletonStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingletonStorage")
            .field("count", &self.singletons.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, PartialEq)]
    struct GameConfig {
        world_size: u32,
        tick_rate: u32,
    }

    #[derive(Debug, Default, PartialEq)]
    struct Time {
        tick: u64,
        delta: f32,
    }

    #[test]
    fn test_singleton_insert_get() {
        let mut storage = SingletonStorage::new();

        storage.insert(GameConfig {
            world_size: 1000,
            tick_rate: 20,
        });

        let config = storage.get::<GameConfig>().unwrap();
        assert_eq!(config.world_size, 1000);
        assert_eq!(config.tick_rate, 20);
    }

    #[test]
    fn test_singleton_get_mut() {
        let mut storage = SingletonStorage::new();

        storage.insert(Time {
            tick: 0,
            delta: 0.0,
        });

        {
            let time = storage.get_mut::<Time>().unwrap();
            time.tick = 100;
            time.delta = 0.05;
        }

        let time = storage.get::<Time>().unwrap();
        assert_eq!(time.tick, 100);
        assert_eq!(time.delta, 0.05);
    }

    #[test]
    fn test_singleton_replace() {
        let mut storage = SingletonStorage::new();

        storage.insert(GameConfig {
            world_size: 500,
            tick_rate: 10,
        });

        let old = storage.insert(GameConfig {
            world_size: 1000,
            tick_rate: 20,
        });

        assert_eq!(
            old,
            Some(GameConfig {
                world_size: 500,
                tick_rate: 10,
            })
        );

        let config = storage.get::<GameConfig>().unwrap();
        assert_eq!(config.world_size, 1000);
    }

    #[test]
    fn test_singleton_get_or_insert() {
        let mut storage = SingletonStorage::new();

        // First call creates default
        let time = storage.get_or_insert::<Time>();
        assert_eq!(time.tick, 0);

        time.tick = 50;

        // Second call returns existing
        let time = storage.get_or_insert::<Time>();
        assert_eq!(time.tick, 50);
    }

    #[test]
    fn test_singleton_remove() {
        let mut storage = SingletonStorage::new();

        storage.insert(GameConfig {
            world_size: 1000,
            tick_rate: 20,
        });

        let removed = storage.remove::<GameConfig>();
        assert!(removed.is_some());
        assert!(!storage.contains::<GameConfig>());
    }
}
