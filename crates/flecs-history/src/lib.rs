//! # flecs-serialize
//!
//! Serialization support for Flecs ECS components.
//!
//! This crate provides:
//! - `SerializeInfo`: A component attached to component entities that provides serialization functions
//! - `SerializableExt`: An extension trait for ergonomic registration of serializable components
//!
//! # Design
//!
//! In Flecs, components are entities. This means we can attach data to component entities
//! themselves. We use this to store serialization function pointers on components that
//! should be serializable.
//!
//! For persistence, use nebari or another database with keys like:
//! `{entity_uuid}:{component_name}` → `{serialized_bytes}`
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
//! // Now you can serialize any Position value
//! let pos = Position { x: 1.0, y: 2.0 };
//! let bytes = serialize_component(&world, &pos).unwrap();
//! let json = serialize_component_json(&world, &pos).unwrap();
//! ```

#![allow(unsafe_code)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]

use core::ffi::c_void;
use std::any::TypeId;

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
        SerializableExt, SerializeError, SerializeInfo, get_serialize_info, is_serializable,
        serialize_component, serialize_component_json,
    };
}

// ════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
    fn test_on_set_observer_with_serializable() {
        let world = World::new();

        // Register Position as serializable
        world.component::<Position>().serializable::<Position>();

        // Track how many times the observer fires and what data it captures
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        // Set up an OnSet observer that uses SerializeInfo to serialize the component
        world
            .component::<Position>()
            .on_set(move |entity, pos: &mut Position| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);

                // Get SerializeInfo from the component entity and serialize
                let world = entity.world();
                let comp_entity = world.component::<Position>().entity();

                if let Some(info) = comp_entity.try_get::<&SerializeInfo>(|s| s.clone()) {
                    let ptr = core::ptr::from_ref(pos).cast::<c_void>();
                    let bytes = (info.to_bytes)(ptr, info.component_size);

                    // Verify we can serialize
                    assert!(!bytes.is_empty());

                    // Verify we can deserialize back
                    let restored: Position = bincode::deserialize(&bytes).unwrap();
                    assert_eq!(*pos, restored);
                } else {
                    panic!("SerializeInfo should be attached!");
                }
            });

        // Create entity and set Position - should trigger observer
        let entity = world.entity();
        entity.set(Position { x: 1.0, y: 2.0 });
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Update Position - should trigger observer again
        entity.set(Position { x: 3.0, y: 4.0 });
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        // Create another entity - should trigger observer
        let entity2 = world.entity();
        entity2.set(Position { x: 5.0, y: 6.0 });
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_observer_can_access_entity_uuid() {
        let world = World::new();

        world.component::<Position>().serializable::<Position>();

        let captured_ids = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured_ids_clone = captured_ids.clone();

        world
            .component::<Position>()
            .on_set(move |entity, _pos: &mut Position| {
                // Capture entity id - in real use, you'd look up UUID component
                let entity_id = entity.id().0;
                captured_ids_clone.lock().unwrap().push(entity_id);
            });

        let e1 = world.entity();
        let e2 = world.entity();

        e1.set(Position { x: 1.0, y: 1.0 });
        e2.set(Position { x: 2.0, y: 2.0 });
        e1.set(Position { x: 3.0, y: 3.0 }); // Update e1 again

        let ids = captured_ids.lock().unwrap();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], e1.id().0);
        assert_eq!(ids[1], e2.id().0);
        assert_eq!(ids[2], e1.id().0);
    }

    #[test]
    fn test_wildcard_observer_for_any_serializable_component() {
        let world = World::new();

        // Register multiple components as serializable
        world.component::<Position>().serializable::<Position>();
        world.component::<Velocity>().serializable::<Velocity>();

        // Track all OnSet events
        let events = Arc::new(std::sync::Mutex::new(Vec::<(u64, String)>::new()));
        let events_clone = events.clone();

        // Create a wildcard observer that fires on ANY component OnSet
        world
            .observer::<flecs::OnSet, ()>()
            .with(flecs::Wildcard::id())
            .each_iter(move |it, index, _| {
                let entity = it.entity(index);
                let component_id = it.event_id();
                let world = it.world();
                let comp_entity = world.entity_from_id(*component_id.id());

                // Check if this component has SerializeInfo
                if comp_entity.try_get::<&SerializeInfo>(|_| ()).is_some() {
                    let comp_name = comp_entity.name();
                    events_clone
                        .lock()
                        .unwrap()
                        .push((entity.id().0, comp_name));
                }
            });

        let e1 = world.entity();
        let e2 = world.entity();

        e1.set(Position { x: 1.0, y: 2.0 });
        e1.set(Velocity { x: 3.0, y: 4.0 });
        e2.set(Position { x: 5.0, y: 6.0 });

        let captured = events.lock().unwrap();
        assert_eq!(captured.len(), 3);
        assert_eq!(captured[0].1, "Position");
        assert_eq!(captured[1].1, "Velocity");
        assert_eq!(captured[2].1, "Position");
    }

    /// UUID component for identifying persistable entities
    #[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Uuid(u128);

    #[test]
    fn test_wildcard_observer_only_persists_entities_with_uuid() {
        let world = World::new();

        // Register components as serializable
        world.component::<Position>().serializable::<Position>();
        world.component::<Uuid>().serializable::<Uuid>();

        // Track persisted events: (uuid, component_name, serialized_bytes)
        let persisted = Arc::new(std::sync::Mutex::new(Vec::<(u128, String, Vec<u8>)>::new()));
        let persisted_clone = persisted.clone();

        // Create a wildcard observer that:
        // 1. Fires on ANY component OnSet
        // 2. Only persists if entity has Uuid
        // 3. Only persists if component has SerializeInfo
        world
            .observer::<flecs::OnSet, ()>()
            .with(flecs::Wildcard::id())
            .each_iter(move |it, index, _| {
                let entity = it.entity(index);
                let component_id = it.event_id();
                let world = it.world();
                let comp_entity = world.entity_from_id(*component_id.id());

                // Check if entity has Uuid
                let Some(uuid) = entity.try_get::<&Uuid>(|u| u.0) else {
                    return; // Entity doesn't have Uuid, skip persistence
                };

                // Check if component has SerializeInfo and serialize
                if let Some(info) = comp_entity.try_get::<&SerializeInfo>(|s| s.clone()) {
                    let comp_name = comp_entity.name();

                    // JANK: Using raw ecs_get_id to get component pointer since wildcard
                    // observer doesn't give us typed access. There might be a better way
                    // to get the component data in a wildcard observer context.
                    let comp_ptr = unsafe {
                        flecs_ecs::sys::ecs_get_id(
                            world.world_ptr(),
                            entity.id().0,
                            *component_id.id(),
                        )
                    };

                    if !comp_ptr.is_null() {
                        // Serialize using the SerializeInfo
                        let bytes = (info.to_bytes)(comp_ptr, info.component_size);

                        persisted_clone
                            .lock()
                            .unwrap()
                            .push((uuid, comp_name, bytes));
                    }
                }
            });

        // Entity WITHOUT Uuid - changes should NOT be persisted
        let e_no_uuid = world.entity();
        e_no_uuid.set(Position { x: 1.0, y: 2.0 });

        // Entity WITH Uuid - changes SHOULD be persisted
        let e_with_uuid = world.entity();
        e_with_uuid.set(Uuid(12345));
        e_with_uuid.set(Position { x: 3.0, y: 4.0 });

        // Another entity with Uuid
        let e_with_uuid2 = world.entity();
        e_with_uuid2.set(Uuid(67890));
        e_with_uuid2.set(Position { x: 5.0, y: 6.0 });

        let captured = persisted.lock().unwrap();

        // Should have 4 events:
        // 1. e_with_uuid sets Uuid (persisted because it now has Uuid)
        // 2. e_with_uuid sets Position (persisted)
        // 3. e_with_uuid2 sets Uuid (persisted)
        // 4. e_with_uuid2 sets Position (persisted)
        // Note: e_no_uuid's Position was NOT persisted
        assert_eq!(captured.len(), 4);

        // Verify the UUIDs match
        assert_eq!(captured[0].0, 12345); // Uuid component itself
        assert_eq!(captured[1].0, 12345); // Position on same entity
        assert_eq!(captured[2].0, 67890); // Uuid on second entity
        assert_eq!(captured[3].0, 67890); // Position on second entity

        // Verify actual serialization worked - deserialize Position back
        let pos_bytes = &captured[1].2;
        let pos: Position = bincode::deserialize(pos_bytes).unwrap();
        assert_eq!(pos, Position { x: 3.0, y: 4.0 });

        let pos2_bytes = &captured[3].2;
        let pos2: Position = bincode::deserialize(pos2_bytes).unwrap();
        assert_eq!(pos2, Position { x: 5.0, y: 6.0 });
    }
}
