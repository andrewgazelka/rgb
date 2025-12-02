//! # flecs-history
//!
//! Serialization and history tracking for Flecs ECS components.
//!
//! This crate provides:
//! - `SerializeInfo`: A component attached to component entities that provides serialization functions
//! - `SerializableExt`: An extension trait for ergonomic registration of serializable components
//! - History tracking: Automatic recording of component changes for entities
//!
//! # Design
//!
//! In Flecs, components are entities. This means we can attach data to component entities
//! themselves. We use this to store serialization function pointers on components that
//! should be serializable.
//!
//! History tracking works by:
//! 1. Setting up hooks for `OnSet` events on tracked components
//! 2. When a component changes, serializing the value and storing it as a history entry
//! 3. History entries are stored as entities with pair relations:
//!    - `(HistoryOf, component_entity)` - which component type
//!    - `(HistoryFor, source_entity)` - which entity the value came from
//!
//! # Example
//!
//! ```ignore
//! use flecs_ecs::prelude::*;
//! use flecs_history::prelude::*;
//!
//! #[derive(Component, Clone, serde::Serialize, serde::Deserialize)]
//! struct Position { x: f32, y: f32 }
//!
//! let world = World::new();
//!
//! // Register Position as serializable (attaches SerializeInfo to Position's component entity)
//! world.component::<Position>().serializable::<Position>();
//!
//! // Set up history tracking
//! let history = HistoryTracker::new(&world);
//!
//! // Enable tracking for Position (sets up OnSet hook)
//! history.track_component::<Position>(&world);
//!
//! // Now any changes to Position will be recorded
//! let entity = world.entity().set(Position { x: 0.0, y: 0.0 });
//! entity.set(Position { x: 1.0, y: 1.0 });
//! entity.set(Position { x: 2.0, y: 2.0 });
//!
//! // Query history
//! for entry in history.get_component_history::<Position>(&world, entity) {
//!     println!("tick {}: {:?}", entry.tick, entry.deserialize::<Position>());
//! }
//! ```

#![allow(unsafe_code)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]

use core::ffi::c_void;
use std::any::TypeId;
use std::sync::{Arc, Mutex};

use flecs_ecs::prelude::*;
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════════════════
// SerializeInfo - attached to component entities
// ════════════════════════════════════════════════════════════════════════════

/// Serialization function pointers stored on component entities.
///
/// This component is attached to component entities (not regular entities) to indicate
/// that the component type is serializable. It stores type-erased function pointers
/// for serializing component data to bytes and JSON.
///
/// # Mathematical Representation
///
/// For a component type `T`:
/// - `to_bytes: T → Vec<u8>` (binary serialization)
/// - `from_bytes: Vec<u8> → T` (binary deserialization)
/// - `to_json: T → serde_json::Value` (JSON serialization)
#[derive(Component, Clone)]
pub struct SerializeInfo {
    /// Serialize component data to bytes.
    /// Takes a pointer to the component data and its size.
    pub to_bytes: fn(*const c_void, usize) -> Vec<u8>,

    /// Deserialize bytes back to component data.
    /// Takes bytes and writes to the destination pointer.
    pub from_bytes: fn(&[u8], *mut c_void),

    /// Serialize component data to JSON (for debugging/dashboard).
    pub to_json: fn(*const c_void) -> serde_json::Value,

    /// Size of the component in bytes.
    pub component_size: usize,

    /// Type ID for runtime type checking.
    pub type_id: TypeId,
}

/// Error type for serialization operations.
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
    #[error("bincode serialization failed: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("json serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("component has no SerializeInfo")]
    NotSerializable,

    #[error("type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
}

// ════════════════════════════════════════════════════════════════════════════
// SerializableExt - extension trait for ergonomic registration
// ════════════════════════════════════════════════════════════════════════════

/// Extension trait for registering serializable components.
///
/// This trait is implemented for `flecs_ecs::core::Component` and allows
/// chaining `.serializable::<T>()` when registering components.
pub trait SerializableExt<'a> {
    /// Mark this component as serializable by attaching `SerializeInfo`.
    ///
    /// The type parameter `T` must match the component type and implement
    /// `Serialize + DeserializeOwned`.
    fn serializable<T>(self) -> Self
    where
        T: Serialize + for<'de> Deserialize<'de> + 'static;
}

impl<'a, C: ComponentId> SerializableExt<'a> for flecs_ecs::core::Component<'a, C> {
    fn serializable<T>(self) -> Self
    where
        T: Serialize + for<'de> Deserialize<'de> + 'static,
    {
        self.entity().set(SerializeInfo {
            to_bytes: |ptr, size| {
                assert_eq!(size, core::mem::size_of::<T>());
                let val = unsafe { &*ptr.cast::<T>() };
                bincode::serialize(val).expect("serialization should not fail")
            },
            from_bytes: |bytes, ptr| {
                let val: T = bincode::deserialize(bytes).expect("deserialization should not fail");
                unsafe { core::ptr::write(ptr.cast::<T>(), val) };
            },
            to_json: |ptr| {
                let val = unsafe { &*ptr.cast::<T>() };
                serde_json::to_value(val).expect("json serialization should not fail")
            },
            component_size: core::mem::size_of::<T>(),
            type_id: TypeId::of::<T>(),
        });
        self
    }
}

// ════════════════════════════════════════════════════════════════════════════
// History Entry - stores a single historical value
// ════════════════════════════════════════════════════════════════════════════

/// A single history entry storing a serialized component value at a point in time.
#[derive(Component, Clone)]
pub struct HistoryEntry {
    /// The tick/frame when this value was recorded.
    pub tick: u64,

    /// The serialized component data.
    pub data: Vec<u8>,

    /// The component entity ID (which component type this is).
    pub component_id: u64,
}

impl HistoryEntry {
    /// Deserialize this entry back to the component type.
    pub fn deserialize<T>(&self) -> Result<T, SerializeError>
    where
        T: for<'de> Deserialize<'de>,
    {
        Ok(bincode::deserialize(&self.data)?)
    }

    /// Convert this entry to a JSON value.
    /// Note: This only works if the data was serialized with bincode from a JSON-serializable type.
    pub fn to_json_raw(&self) -> serde_json::Value {
        serde_json::from_slice(&self.data).unwrap_or(serde_json::Value::Null)
    }
}

/// Relation tag: history entry is for this component type.
/// Used as: entity.add((HistoryOf, component_entity))
#[derive(Component)]
pub struct HistoryOf;

/// Relation tag: history entry belongs to this entity.
/// Used as: entity.add((HistoryFor, source_entity))
#[derive(Component)]
pub struct HistoryFor;

// ════════════════════════════════════════════════════════════════════════════
// History Tracker - manages history recording
// ════════════════════════════════════════════════════════════════════════════

/// Shared state for history tracking across observers.
#[derive(Clone)]
struct HistoryState {
    /// Current tick counter.
    tick: Arc<Mutex<u64>>,

    /// Maximum number of entries per (entity, component) pair.
    #[allow(dead_code)]
    max_entries: usize,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            tick: Arc::new(Mutex::new(0)),
            max_entries: 1000,
        }
    }
}

/// History tracker that records component changes.
///
/// Create one of these and call `track_component::<T>()` for each component
/// type you want to track. The tracker will automatically record changes
/// to any component that has `SerializeInfo` attached.
pub struct HistoryTracker {
    state: HistoryState,
}

impl HistoryTracker {
    /// Create a new history tracker with default settings.
    pub fn new(world: &World) -> Self {
        Self::with_max_entries(world, 1000)
    }

    /// Create a new history tracker with a custom max entries limit.
    pub fn with_max_entries(world: &World, max_entries: usize) -> Self {
        let state = HistoryState {
            max_entries,
            ..Default::default()
        };

        // Register our components
        world.component::<SerializeInfo>();
        world.component::<HistoryEntry>();
        world.component::<HistoryOf>();
        world.component::<HistoryFor>();

        Self { state }
    }

    /// Enable history tracking for a specific component type.
    ///
    /// This sets up an `on_set` hook for the component. The component must
    /// already be registered with `.serializable()`.
    ///
    /// # Panics
    ///
    /// Panics if the component doesn't have `SerializeInfo` attached.
    pub fn track_component<T>(&self, world: &World)
    where
        T: ComponentId + 'static,
    {
        let state = self.state.clone();

        // Get the component entity to verify it has SerializeInfo
        let comp_entity = world.component::<T>().entity();
        let has_serialize_info = comp_entity.try_get::<&SerializeInfo>(|_| ()).is_some();

        assert!(
            has_serialize_info,
            "Component {} must be registered with .serializable() before tracking",
            core::any::type_name::<T>()
        );

        let comp_id = comp_entity.id().0;

        // Set up an OnSet hook for this component
        world.component::<T>().on_set(
            move |entity: EntityView<'_>, component: &mut <T as ComponentId>::UnderlyingType| {
                // Get current tick
                let tick = {
                    let guard = state.tick.lock().unwrap();
                    *guard
                };

                // Serialize the component value using the SerializeInfo
                // We need to get SerializeInfo from the component entity
                let world = entity.world();
                let comp_entity = world.component::<T>().entity();

                if let Some(info) = comp_entity.try_get::<&SerializeInfo>(|s| s.clone()) {
                    let ptr = core::ptr::from_ref(component).cast::<c_void>();
                    let bytes = (info.to_bytes)(ptr, info.component_size);

                    // Create a history entry as a new entity with pair relations
                    world
                        .entity()
                        .set(HistoryEntry {
                            tick,
                            data: bytes,
                            component_id: comp_id,
                        })
                        .add((HistoryOf, comp_entity))
                        .add((HistoryFor, entity));
                }
            },
        );
    }

    /// Advance the tick counter.
    pub fn advance_tick(&self) {
        let mut guard = self.state.tick.lock().unwrap();
        *guard += 1;
    }

    /// Get the current tick.
    pub fn current_tick(&self) -> u64 {
        *self.state.tick.lock().unwrap()
    }

    /// Set the current tick.
    pub fn set_tick(&self, tick: u64) {
        *self.state.tick.lock().unwrap() = tick;
    }

    /// Query all history entries for a specific entity and component type.
    pub fn get_component_history<T: ComponentId>(
        &self,
        world: &World,
        entity: impl Into<Entity>,
    ) -> Vec<HistoryEntry> {
        let entity = entity.into();
        let comp_entity = world.component::<T>().entity();
        self.get_history_for_component_id(world, entity, comp_entity)
    }

    /// Query all history entries for a specific entity and component ID.
    pub fn get_history_for_component_id(
        &self,
        world: &World,
        entity: impl Into<Entity>,
        component: impl Into<Entity>,
    ) -> Vec<HistoryEntry> {
        let entity = entity.into();
        let component = component.into();

        let mut results = Vec::new();

        // Query: HistoryEntry with (HistoryOf, component) and (HistoryFor, entity)
        world
            .query::<&HistoryEntry>()
            .with((HistoryOf, component))
            .with((HistoryFor, entity))
            .build()
            .each(|entry| {
                results.push(entry.clone());
            });

        // Sort by tick
        results.sort_by_key(|e| e.tick);
        results
    }

    /// Query all history entries for a specific entity (all components).
    pub fn get_entity_history(
        &self,
        world: &World,
        entity: impl Into<Entity>,
    ) -> Vec<HistoryEntry> {
        let entity = entity.into();
        let mut results = Vec::new();

        world
            .query::<&HistoryEntry>()
            .with((HistoryFor, entity))
            .build()
            .each(|entry| {
                results.push(entry.clone());
            });

        results.sort_by_key(|e| e.tick);
        results
    }

    /// Get the value of a component at a specific tick.
    ///
    /// Returns the most recent value at or before the given tick.
    pub fn get_at_tick<T>(&self, world: &World, entity: impl Into<Entity>, tick: u64) -> Option<T>
    where
        T: ComponentId + for<'de> Deserialize<'de>,
    {
        let history = self.get_component_history::<T>(world, entity);

        // Find the most recent entry at or before the tick
        history
            .into_iter()
            .rev()
            .find(|e| e.tick <= tick)
            .and_then(|e| e.deserialize().ok())
    }

    /// Get history entries in a tick range (inclusive).
    pub fn get_in_range<T: ComponentId>(
        &self,
        world: &World,
        entity: impl Into<Entity>,
        start_tick: u64,
        end_tick: u64,
    ) -> Vec<HistoryEntry> {
        self.get_component_history::<T>(world, entity)
            .into_iter()
            .filter(|e| e.tick >= start_tick && e.tick <= end_tick)
            .collect()
    }

    /// Clear all history for a specific entity.
    pub fn clear_entity_history(&self, world: &World, entity: impl Into<Entity>) {
        let entity = entity.into();

        // Find all history entries for this entity and delete them
        let mut to_delete = Vec::new();

        world
            .query::<&HistoryEntry>()
            .with((HistoryFor, entity))
            .build()
            .each_entity(|e, _| {
                to_delete.push(e.id());
            });

        for id in to_delete {
            world.entity_from_id(id).destruct();
        }
    }

    /// Clear all history.
    pub fn clear_all_history(&self, world: &World) {
        let mut to_delete = Vec::new();

        world.query::<&HistoryEntry>().build().each_entity(|e, _| {
            to_delete.push(e.id());
        });

        for id in to_delete {
            world.entity_from_id(id).destruct();
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Utility functions
// ════════════════════════════════════════════════════════════════════════════

/// Check if a component has SerializeInfo attached.
pub fn is_serializable<T: ComponentId>(world: &World) -> bool {
    world
        .component::<T>()
        .entity()
        .try_get::<&SerializeInfo>(|_| ())
        .is_some()
}

/// Get the SerializeInfo for a component, if it exists.
pub fn get_serialize_info<T: ComponentId>(world: &World) -> Option<SerializeInfo> {
    world
        .component::<T>()
        .entity()
        .try_get::<&SerializeInfo>(|s| s.clone())
}

/// Serialize a component value to bytes.
pub fn serialize_component<T>(world: &World, value: &T) -> Result<Vec<u8>, SerializeError>
where
    T: ComponentId + Serialize,
{
    let info = get_serialize_info::<T>(world).ok_or(SerializeError::NotSerializable)?;
    let ptr = core::ptr::from_ref(value).cast::<c_void>();
    Ok((info.to_bytes)(ptr, info.component_size))
}

/// Serialize a component value to JSON.
pub fn serialize_component_json<T>(
    world: &World,
    value: &T,
) -> Result<serde_json::Value, SerializeError>
where
    T: ComponentId + Serialize,
{
    let info = get_serialize_info::<T>(world).ok_or(SerializeError::NotSerializable)?;
    let ptr = core::ptr::from_ref(value).cast::<c_void>();
    Ok((info.to_json)(ptr))
}

// ════════════════════════════════════════════════════════════════════════════
// Prelude
// ════════════════════════════════════════════════════════════════════════════

pub mod prelude {
    pub use crate::{
        HistoryEntry, HistoryFor, HistoryOf, HistoryTracker, SerializableExt, SerializeError,
        SerializeInfo, get_serialize_info, is_serializable, serialize_component,
        serialize_component_json,
    };
}

// ════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[derive(Component, Clone)]
    struct NotSerializable {
        #[allow(dead_code)]
        handle: u64,
    }

    #[test]
    fn test_serialize_info_registration() {
        let world = World::new();

        // Register Position as serializable
        world.component::<Position>().serializable::<Position>();

        // Check that SerializeInfo was attached
        assert!(is_serializable::<Position>(&world));

        // Velocity is not registered as serializable
        world.component::<Velocity>();
        assert!(!is_serializable::<Velocity>(&world));
    }

    #[test]
    fn test_serialization() {
        let world = World::new();
        world.component::<Position>().serializable::<Position>();

        let pos = Position { x: 1.0, y: 2.0 };

        // Serialize to bytes
        let bytes = serialize_component(&world, &pos).unwrap();
        assert!(!bytes.is_empty());

        // Deserialize back
        let restored: Position = bincode::deserialize(&bytes).unwrap();
        assert_eq!(pos, restored);

        // Serialize to JSON
        let json = serialize_component_json(&world, &pos).unwrap();
        assert_eq!(json["x"], 1.0);
        assert_eq!(json["y"], 2.0);
    }

    #[test]
    fn test_history_tracking() {
        let world = World::new();

        // Register Position as serializable
        world.component::<Position>().serializable::<Position>();

        // Create history tracker and enable tracking for Position
        let history = HistoryTracker::new(&world);
        history.track_component::<Position>(&world);

        // Create an entity and set Position multiple times
        let entity = world.entity();

        history.set_tick(0);
        entity.set(Position { x: 0.0, y: 0.0 });

        history.set_tick(1);
        entity.set(Position { x: 1.0, y: 1.0 });

        history.set_tick(2);
        entity.set(Position { x: 2.0, y: 2.0 });

        // Query history
        let entries = history.get_component_history::<Position>(&world, entity);
        assert_eq!(entries.len(), 3);

        // Verify values
        let pos0: Position = entries[0].deserialize().unwrap();
        assert_eq!(pos0, Position { x: 0.0, y: 0.0 });

        let pos1: Position = entries[1].deserialize().unwrap();
        assert_eq!(pos1, Position { x: 1.0, y: 1.0 });

        let pos2: Position = entries[2].deserialize().unwrap();
        assert_eq!(pos2, Position { x: 2.0, y: 2.0 });
    }

    #[test]
    fn test_get_at_tick() {
        let world = World::new();
        world.component::<Position>().serializable::<Position>();

        let history = HistoryTracker::new(&world);
        history.track_component::<Position>(&world);

        let entity = world.entity();

        history.set_tick(0);
        entity.set(Position { x: 0.0, y: 0.0 });

        history.set_tick(5);
        entity.set(Position { x: 5.0, y: 5.0 });

        history.set_tick(10);
        entity.set(Position { x: 10.0, y: 10.0 });

        // Query at specific ticks
        let at_0: Position = history.get_at_tick(&world, entity, 0).unwrap();
        assert_eq!(at_0.x, 0.0);

        let at_3: Position = history.get_at_tick(&world, entity, 3).unwrap();
        assert_eq!(at_3.x, 0.0); // Most recent at or before tick 3

        let at_7: Position = history.get_at_tick(&world, entity, 7).unwrap();
        assert_eq!(at_7.x, 5.0); // Most recent at or before tick 7

        let at_10: Position = history.get_at_tick(&world, entity, 10).unwrap();
        assert_eq!(at_10.x, 10.0);
    }

    #[test]
    fn test_clear_history() {
        let world = World::new();
        world.component::<Position>().serializable::<Position>();

        let history = HistoryTracker::new(&world);
        history.track_component::<Position>(&world);

        let entity = world.entity();
        entity.set(Position { x: 1.0, y: 1.0 });
        entity.set(Position { x: 2.0, y: 2.0 });

        assert_eq!(
            history
                .get_component_history::<Position>(&world, entity)
                .len(),
            2
        );

        history.clear_entity_history(&world, entity);

        assert_eq!(
            history
                .get_component_history::<Position>(&world, entity)
                .len(),
            0
        );
    }

    #[test]
    fn test_multiple_entities() {
        let world = World::new();
        world.component::<Position>().serializable::<Position>();

        let history = HistoryTracker::new(&world);
        history.track_component::<Position>(&world);

        let entity1 = world.entity();
        let entity2 = world.entity();

        history.set_tick(0);
        entity1.set(Position { x: 1.0, y: 1.0 });
        entity2.set(Position { x: 100.0, y: 100.0 });

        history.set_tick(1);
        entity1.set(Position { x: 2.0, y: 2.0 });
        entity2.set(Position { x: 200.0, y: 200.0 });

        // Each entity should have its own history
        let entries1 = history.get_component_history::<Position>(&world, entity1);
        let entries2 = history.get_component_history::<Position>(&world, entity2);

        assert_eq!(entries1.len(), 2);
        assert_eq!(entries2.len(), 2);

        let pos1: Position = entries1[0].deserialize().unwrap();
        let pos2: Position = entries2[0].deserialize().unwrap();

        assert_eq!(pos1.x, 1.0);
        assert_eq!(pos2.x, 100.0);
    }

    #[test]
    fn test_entity_history_all_components() {
        let world = World::new();
        world.component::<Position>().serializable::<Position>();
        world.component::<Velocity>().serializable::<Velocity>();

        let history = HistoryTracker::new(&world);
        history.track_component::<Position>(&world);
        history.track_component::<Velocity>(&world);

        let entity = world.entity();

        history.set_tick(0);
        entity.set(Position { x: 1.0, y: 1.0 });
        entity.set(Velocity { x: 10.0, y: 10.0 });

        // Get all history for entity (both components)
        let all_entries = history.get_entity_history(&world, entity);
        assert_eq!(all_entries.len(), 2);
    }
}
