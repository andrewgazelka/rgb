//! Component type registration and metadata.
//!
//! Components are data types that can be attached to entities.
//! Each component type has a unique ID and associated metadata
//! for runtime type operations.

use std::{
    alloc::Layout,
    any::TypeId,
    collections::HashMap,
    fmt,
    sync::atomic::{AtomicU32, Ordering},
};

/// Marker trait for types that can be used as components.
///
/// # Safety
///
/// Types implementing this trait must be safe to move in memory
/// (no self-referential pointers).
///
/// # Example
///
/// ```ignore
/// #[derive(Component)]
/// struct Position { x: f32, y: f32, z: f32 }
/// ```
pub trait Component: Send + Sync + 'static {}

// Blanket implementation for all suitable types
impl<T: Send + Sync + 'static> Component for T {}

/// Unique identifier for a component type.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentId(u32);

impl ComponentId {
    /// Create a component ID from a raw value.
    #[must_use]
    pub const fn from_raw(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    #[must_use]
    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ComponentId({})", self.0)
    }
}

/// Runtime information about a component type.
#[derive(Clone)]
pub struct ComponentInfo {
    /// Unique ID for this component type.
    id: ComponentId,
    /// Type name for debugging.
    name: &'static str,
    /// Memory layout of the component.
    layout: Layout,
    /// Function to drop a component in place.
    drop_fn: Option<unsafe fn(*mut u8)>,
    /// Rust TypeId for type checking.
    type_id: TypeId,
}

impl ComponentInfo {
    /// Create component info for a concrete type.
    #[must_use]
    pub fn of<T: Component>(id: ComponentId) -> Self {
        Self {
            id,
            name: std::any::type_name::<T>(),
            layout: Layout::new::<T>(),
            drop_fn: if std::mem::needs_drop::<T>() {
                Some(|ptr| unsafe { std::ptr::drop_in_place(ptr.cast::<T>()) })
            } else {
                None
            },
            type_id: TypeId::of::<T>(),
        }
    }

    /// Get the component ID.
    #[must_use]
    pub const fn id(&self) -> ComponentId {
        self.id
    }

    /// Get the component type name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Get the memory layout.
    #[must_use]
    pub const fn layout(&self) -> Layout {
        self.layout
    }

    /// Get the size in bytes.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.layout.size()
    }

    /// Get the alignment requirement.
    #[must_use]
    pub const fn align(&self) -> usize {
        self.layout.align()
    }

    /// Check if the component needs drop.
    #[must_use]
    pub const fn needs_drop(&self) -> bool {
        self.drop_fn.is_some()
    }

    /// Drop a component at the given pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must point to a valid, initialized instance of this component type.
    /// - The memory at `ptr` must not be accessed after this call.
    pub unsafe fn drop_in_place(&self, ptr: *mut u8) {
        if let Some(drop_fn) = self.drop_fn {
            unsafe { drop_fn(ptr) };
        }
    }

    /// Check if this info is for the given type.
    #[must_use]
    pub fn is<T: 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }
}

impl fmt::Debug for ComponentInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentInfo")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("size", &self.layout.size())
            .field("align", &self.layout.align())
            .finish()
    }
}

/// Global counter for generating unique component IDs.
static NEXT_COMPONENT_ID: AtomicU32 = AtomicU32::new(0);

/// Registry for component types.
///
/// Maps Rust types to `ComponentId`s and stores metadata about each type.
/// Thread-safe for registration, but typically populated at startup.
#[derive(Default)]
pub struct ComponentRegistry {
    /// Map from TypeId to ComponentId.
    type_to_id: HashMap<TypeId, ComponentId>,
    /// Component info indexed by ComponentId.
    infos: Vec<ComponentInfo>,
}

impl ComponentRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a component type and return its ID.
    ///
    /// If the type is already registered, returns the existing ID.
    pub fn register<T: Component>(&mut self) -> ComponentId {
        let type_id = TypeId::of::<T>();

        if let Some(&id) = self.type_to_id.get(&type_id) {
            return id;
        }

        let id = ComponentId(NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed));
        let info = ComponentInfo::of::<T>(id);

        self.type_to_id.insert(type_id, id);

        // Ensure infos vec is large enough
        let idx = id.as_raw() as usize;
        if idx >= self.infos.len() {
            self.infos.resize(idx + 1, info.clone());
        }
        self.infos[idx] = info;

        id
    }

    /// Get the component ID for a type, if registered.
    #[must_use]
    pub fn get_id<T: Component>(&self) -> Option<ComponentId> {
        self.type_to_id.get(&TypeId::of::<T>()).copied()
    }

    /// Get the component ID for a TypeId, if registered.
    #[must_use]
    pub fn get_id_by_type_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.type_to_id.get(&type_id).copied()
    }

    /// Get component info by ID.
    #[must_use]
    pub fn get_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        self.infos.get(id.as_raw() as usize)
    }

    /// Get the number of registered components.
    #[must_use]
    pub fn len(&self) -> usize {
        self.type_to_id.len()
    }

    /// Check if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.type_to_id.is_empty()
    }

    /// Iterate over all registered component infos.
    pub fn iter(&self) -> impl Iterator<Item = &ComponentInfo> {
        self.infos.iter()
    }
}

impl fmt::Debug for ComponentRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field("count", &self.len())
            .field("components", &self.infos)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        x: f32,
        y: f32,
    }

    struct Velocity {
        x: f32,
        y: f32,
    }

    struct Name(String);

    #[test]
    fn test_component_registration() {
        let mut registry = ComponentRegistry::new();

        let pos_id = registry.register::<Position>();
        let vel_id = registry.register::<Velocity>();

        assert_ne!(pos_id, vel_id);
        assert_eq!(registry.get_id::<Position>(), Some(pos_id));
        assert_eq!(registry.get_id::<Velocity>(), Some(vel_id));
    }

    #[test]
    fn test_component_info() {
        let mut registry = ComponentRegistry::new();

        let pos_id = registry.register::<Position>();
        let info = registry.get_info(pos_id).unwrap();

        assert_eq!(info.size(), std::mem::size_of::<Position>());
        assert_eq!(info.align(), std::mem::align_of::<Position>());
        assert!(!info.needs_drop());
        assert!(info.is::<Position>());
    }

    #[test]
    fn test_component_with_drop() {
        let mut registry = ComponentRegistry::new();

        let name_id = registry.register::<Name>();
        let info = registry.get_info(name_id).unwrap();

        assert!(info.needs_drop());
    }

    #[test]
    fn test_idempotent_registration() {
        let mut registry = ComponentRegistry::new();

        let id1 = registry.register::<Position>();
        let id2 = registry.register::<Position>();

        assert_eq!(id1, id2);
        assert_eq!(registry.len(), 1);
    }
}
