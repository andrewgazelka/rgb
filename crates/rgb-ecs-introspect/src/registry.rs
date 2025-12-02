//! Type-erased registry for introspectable components.

use std::alloc::Layout;
use std::any::TypeId;
use std::collections::HashMap;

use rgb_ecs::{ComponentId, World};

use crate::{IntrospectError, Introspectable};

/// Type-erased information about an introspectable component.
pub struct IntrospectInfo {
    /// Component ID in the ECS.
    pub component_id: ComponentId,
    /// Rust TypeId for type checking.
    pub type_id: TypeId,
    /// Short type name (e.g., "Position").
    pub name: &'static str,
    /// Full type name (e.g., "mc_server_runner::components::Position").
    pub full_name: &'static str,
    /// Memory layout of the component.
    pub layout: Layout,
    /// Whether this is an opaque component.
    pub is_opaque: bool,
    /// JSON schema for the component.
    pub schema: Option<serde_json::Value>,
    /// Function to serialize component to JSON from raw pointer.
    serialize_fn: SerializeFn,
    /// Function to deserialize JSON to component bytes.
    deserialize_fn: DeserializeFn,
    /// Function to get opaque info summary from raw pointer.
    opaque_info_fn: OpaqueInfoFn,
}

type SerializeFn = fn(*const u8) -> serde_json::Value;
type DeserializeFn = fn(serde_json::Value) -> Result<AlignedBuffer, IntrospectError>;
type OpaqueInfoFn = fn(*const u8) -> Option<String>;

/// Public fields for IntrospectInfo
impl IntrospectInfo {
    /// Component ID accessor
    pub fn id(&self) -> ComponentId {
        self.component_id
    }

    /// Size of the component in bytes
    pub fn size(&self) -> usize {
        self.layout.size()
    }
}

impl IntrospectInfo {
    /// Create info for an introspectable type.
    pub fn of<T: Introspectable>(component_id: ComponentId) -> Self {
        Self {
            component_id,
            type_id: TypeId::of::<T>(),
            name: T::type_name(),
            full_name: T::full_type_name(),
            layout: Layout::new::<T>(),
            is_opaque: T::is_opaque(),
            schema: T::schema(),
            serialize_fn: |ptr| {
                // SAFETY: Caller ensures ptr points to valid T
                let value: &T = unsafe { &*(ptr.cast::<T>()) };
                value.to_json()
            },
            deserialize_fn: |json| {
                let value = T::from_json(json)?;
                let layout = Layout::new::<T>();
                let mut buffer = AlignedBuffer::new(layout);
                // SAFETY: buffer is properly sized and aligned for T
                unsafe {
                    core::ptr::write(buffer.as_mut_ptr().cast::<T>(), value);
                }
                Ok(buffer)
            },
            opaque_info_fn: |ptr| {
                // SAFETY: Caller ensures ptr points to valid T
                let value: &T = unsafe { &*(ptr.cast::<T>()) };
                value.opaque_info()
            },
        }
    }

    /// Get opaque info summary for a component from an entity.
    ///
    /// Returns None if the entity doesn't have this component or no info available.
    pub fn get_opaque_info(&self, world: &World, entity: rgb_ecs::Entity) -> Option<String> {
        let ptr = world.get_raw_ptr(entity, self.type_id)?;
        (self.opaque_info_fn)(ptr)
    }

    /// Serialize a component from a raw pointer to JSON.
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid, initialized instance of the component type.
    pub unsafe fn serialize(&self, ptr: *const u8) -> serde_json::Value {
        (self.serialize_fn)(ptr)
    }

    /// Deserialize JSON to component bytes.
    pub fn deserialize(&self, json: serde_json::Value) -> Result<AlignedBuffer, IntrospectError> {
        (self.deserialize_fn)(json)
    }

    /// Get component as JSON from an entity.
    ///
    /// Returns None if the entity doesn't have this component.
    pub fn get_json(&self, world: &World, entity: rgb_ecs::Entity) -> Option<serde_json::Value> {
        let ptr = world.get_raw_ptr(entity, self.type_id)?;
        // SAFETY: get_raw_ptr returns a valid pointer to the component
        Some(unsafe { self.serialize(ptr) })
    }

    /// Set component from JSON on an entity.
    ///
    /// Returns an error if deserialization fails or the component can't be set.
    pub fn set_json(
        &self,
        world: &mut World,
        entity: rgb_ecs::Entity,
        json: &serde_json::Value,
    ) -> Result<(), IntrospectError> {
        let buffer = self.deserialize(json.clone())?;

        // SAFETY: buffer contains valid component data matching the component's layout
        let success = unsafe { world.update_raw(entity, self.component_id, buffer.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(IntrospectError::ComponentNotFound(self.name.to_string()))
        }
    }
}

/// Buffer with proper alignment for component storage.
pub struct AlignedBuffer {
    data: Box<[u8]>,
    layout: Layout,
}

impl AlignedBuffer {
    /// Create a new buffer with the given layout.
    pub fn new(layout: Layout) -> Self {
        let data = vec![0u8; layout.size()].into_boxed_slice();
        Self { data, layout }
    }

    /// Get a pointer to the buffer data.
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Get a mutable pointer to the buffer data.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Get the layout of the buffer.
    pub fn layout(&self) -> Layout {
        self.layout
    }
}

/// Registry of introspectable component types.
///
/// Maps component types to their serialization/deserialization functions.
#[derive(Default)]
pub struct IntrospectRegistry {
    /// ComponentId -> IntrospectInfo
    by_id: HashMap<ComponentId, IntrospectInfo>,
    /// Short name -> ComponentId for API lookups
    by_name: HashMap<String, ComponentId>,
}

impl IntrospectRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an introspectable component type.
    ///
    /// The component must already be registered in the world's component registry.
    pub fn register<T: Introspectable>(&mut self, world: &World) {
        let Some(comp_id) = world.component_id::<T>() else {
            return; // Component not registered in world
        };

        let info = IntrospectInfo::of::<T>(comp_id);
        let name = info.name.to_string();

        self.by_id.insert(comp_id, info);
        self.by_name.insert(name, comp_id);
    }

    /// Get introspect info by component ID.
    #[must_use]
    pub fn get(&self, id: ComponentId) -> Option<&IntrospectInfo> {
        self.by_id.get(&id)
    }

    /// Get introspect info by short type name.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&IntrospectInfo> {
        self.by_name.get(name).and_then(|id| self.by_id.get(id))
    }

    /// Get component ID by short type name.
    #[must_use]
    pub fn component_id(&self, name: &str) -> Option<ComponentId> {
        self.by_name.get(name).copied()
    }

    /// Iterate over all registered introspectable components.
    pub fn iter(&self) -> impl Iterator<Item = &IntrospectInfo> {
        self.by_id.values()
    }

    /// Get the number of registered components.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Check if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}
