//! Archetype storage - tables of entities with identical component layouts.
//!
//! An archetype represents a unique combination of component types.
//! All entities with the same set of components are stored together
//! for cache-efficient iteration.

use std::{collections::HashMap, fmt};

use hashbrown::HashSet;
use smallvec::SmallVec;

use crate::{
    component::{ComponentId, ComponentRegistry},
    entity::Entity,
    storage::Column,
};

/// Unique identifier for an archetype.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    /// The empty archetype (no components).
    pub const EMPTY: Self = Self(0);

    /// Create an archetype ID from a raw value.
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

impl fmt::Debug for ArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArchetypeId({})", self.0)
    }
}

/// An archetype - a table storing entities with the same component layout.
pub struct Archetype {
    /// Unique identifier for this archetype.
    id: ArchetypeId,
    /// Sorted list of component IDs in this archetype.
    components: SmallVec<[ComponentId; 8]>,
    /// Component columns, indexed in same order as `components`.
    columns: Vec<Column>,
    /// Map from ComponentId to column index for fast lookup.
    component_indices: HashMap<ComponentId, usize>,
    /// Entities stored in this archetype.
    entities: Vec<Entity>,
}

impl Archetype {
    /// Create a new archetype with the given component types.
    ///
    /// Components will be sorted by ID for consistent ordering.
    pub fn new(
        id: ArchetypeId,
        component_ids: &[ComponentId],
        registry: &ComponentRegistry,
    ) -> Self {
        let mut components: SmallVec<[ComponentId; 8]> = component_ids.iter().copied().collect();
        components.sort_unstable();

        let mut component_indices = HashMap::with_capacity(components.len());
        let mut columns = Vec::with_capacity(components.len());

        for (idx, &comp_id) in components.iter().enumerate() {
            let info = registry
                .get_info(comp_id)
                .expect("Component must be registered");
            component_indices.insert(comp_id, idx);
            columns.push(Column::new(info.clone()));
        }

        Self {
            id,
            components,
            columns,
            component_indices,
            entities: Vec::new(),
        }
    }

    /// Create the empty archetype (no components).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            id: ArchetypeId::EMPTY,
            components: SmallVec::new(),
            columns: Vec::new(),
            component_indices: HashMap::new(),
            entities: Vec::new(),
        }
    }

    /// Get the archetype ID.
    #[must_use]
    pub const fn id(&self) -> ArchetypeId {
        self.id
    }

    /// Get the component IDs in this archetype (sorted).
    #[must_use]
    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }

    /// Check if this archetype contains a component type.
    #[must_use]
    pub fn contains(&self, component_id: ComponentId) -> bool {
        self.component_indices.contains_key(&component_id)
    }

    /// Get the number of entities in this archetype.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if the archetype is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Get the entities in this archetype.
    #[must_use]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Get the column index for a component type.
    #[must_use]
    pub fn column_index(&self, component_id: ComponentId) -> Option<usize> {
        self.component_indices.get(&component_id).copied()
    }

    /// Get a column by component ID.
    #[must_use]
    pub fn column(&self, component_id: ComponentId) -> Option<&Column> {
        self.column_index(component_id)
            .map(|idx| &self.columns[idx])
    }

    /// Get a mutable column by component ID.
    #[must_use]
    pub fn column_mut(&mut self, component_id: ComponentId) -> Option<&mut Column> {
        self.column_index(component_id)
            .map(|idx| &mut self.columns[idx])
    }

    /// Get a column by index.
    #[must_use]
    pub fn column_by_index(&self, index: usize) -> Option<&Column> {
        self.columns.get(index)
    }

    /// Get a raw pointer to component data at a specific row.
    ///
    /// # Safety
    ///
    /// The caller must ensure the column index and row are valid.
    #[must_use]
    pub fn column_ptr(&self, col_idx: usize, row: usize) -> *const u8 {
        // SAFETY: Caller ensures col_idx and row are valid
        unsafe { self.columns[col_idx].get_unchecked_raw(row) }
    }

    /// Get a mutable column by index.
    #[must_use]
    pub fn column_by_index_mut(&mut self, index: usize) -> Option<&mut Column> {
        self.columns.get_mut(index)
    }

    /// Allocate space for a new entity and return its row index.
    ///
    /// Does NOT initialize component data - caller must write to columns.
    pub fn allocate(&mut self, entity: Entity) -> usize {
        let row = self.entities.len();
        self.entities.push(entity);
        row
    }

    /// Remove an entity by row index using swap-remove.
    ///
    /// Returns the entity that was swapped into this row (if any).
    ///
    /// # Safety
    ///
    /// - `row` must be a valid row index.
    /// - Caller must have already removed component data from columns.
    pub unsafe fn deallocate(&mut self, row: usize) -> Option<Entity> {
        debug_assert!(row < self.entities.len());

        let last_row = self.entities.len() - 1;
        self.entities.swap_remove(row);

        if row < last_row {
            Some(self.entities[row])
        } else {
            None
        }
    }

    /// Set a component value for an entity at the given row.
    ///
    /// # Safety
    ///
    /// - `row` must be less than `len()` (after allocate) or equal to `len()` (during initial setup).
    /// - `T` must match the component type at the given column.
    pub unsafe fn set_component<T: 'static>(
        &mut self,
        component_id: ComponentId,
        row: usize,
        value: T,
    ) {
        let col_idx = self
            .component_indices
            .get(&component_id)
            .expect("Component not in archetype");

        let column = &mut self.columns[*col_idx];

        if row == column.len() {
            // Adding new row
            column.push(value);
        } else {
            // Updating existing row
            // SAFETY: Caller ensures row is valid and type matches
            unsafe {
                *column.get_unchecked_mut::<T>(row) = value;
            }
        }
    }

    /// Get a component reference for an entity at the given row.
    ///
    /// # Safety
    ///
    /// - `row` must be a valid row index.
    /// - `T` must match the component type.
    #[must_use]
    pub unsafe fn get_component<T: 'static>(
        &self,
        component_id: ComponentId,
        row: usize,
    ) -> Option<&T> {
        let col_idx = self.component_indices.get(&component_id)?;
        // SAFETY: Caller ensures row is valid and type matches
        Some(unsafe { self.columns[*col_idx].get_unchecked::<T>(row) })
    }

    /// Get a mutable component reference for an entity at the given row.
    ///
    /// # Safety
    ///
    /// - `row` must be a valid row index.
    /// - `T` must match the component type.
    /// - No other references to this component may exist.
    #[must_use]
    pub unsafe fn get_component_mut<T: 'static>(
        &mut self,
        component_id: ComponentId,
        row: usize,
    ) -> Option<&mut T> {
        let col_idx = self.component_indices.get(&component_id)?;
        // SAFETY: Caller ensures row is valid, type matches, and no aliasing
        Some(unsafe { self.columns[*col_idx].get_unchecked_mut::<T>(row) })
    }

    /// Set a component value from raw bytes at the given row.
    ///
    /// # Safety
    ///
    /// - `row` must be a valid row index (less than `len()`).
    /// - `src` must point to valid, initialized component data matching the component type.
    /// - The layout of the source data must match the component's layout.
    pub unsafe fn set_component_raw(
        &mut self,
        component_id: ComponentId,
        row: usize,
        src: *const u8,
    ) -> bool {
        let Some(&col_idx) = self.component_indices.get(&component_id) else {
            return false;
        };

        // SAFETY: Caller ensures row is valid and src points to valid component data
        unsafe {
            self.columns[col_idx].set_raw(row, src);
        }
        true
    }

    /// Reserve capacity in all columns.
    pub fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
        for column in &mut self.columns {
            column.reserve(additional);
        }
    }
}

impl fmt::Debug for Archetype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Archetype")
            .field("id", &self.id)
            .field("components", &self.components)
            .field("entity_count", &self.entities.len())
            .finish()
    }
}

/// Storage for all archetypes in a world.
pub struct ArchetypeStorage {
    /// All archetypes.
    archetypes: Vec<Archetype>,
    /// Map from component set to archetype ID.
    /// Key is a sorted set of component IDs.
    archetype_map: HashMap<SmallVec<[ComponentId; 8]>, ArchetypeId>,
}

impl Default for ArchetypeStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchetypeStorage {
    /// Create new archetype storage with the empty archetype.
    #[must_use]
    pub fn new() -> Self {
        let mut storage = Self {
            archetypes: Vec::new(),
            archetype_map: HashMap::new(),
        };

        // Create empty archetype at index 0
        let empty_key: SmallVec<[ComponentId; 8]> = SmallVec::new();
        storage.archetypes.push(Archetype::empty());
        storage.archetype_map.insert(empty_key, ArchetypeId::EMPTY);

        storage
    }

    /// Get or create an archetype for the given component set.
    pub fn get_or_create(
        &mut self,
        component_ids: &[ComponentId],
        registry: &ComponentRegistry,
    ) -> ArchetypeId {
        let mut key: SmallVec<[ComponentId; 8]> = component_ids.iter().copied().collect();
        key.sort_unstable();

        if let Some(&id) = self.archetype_map.get(&key) {
            return id;
        }

        let id = ArchetypeId::from_raw(self.archetypes.len() as u32);
        let archetype = Archetype::new(id, component_ids, registry);

        self.archetypes.push(archetype);
        self.archetype_map.insert(key, id);

        id
    }

    /// Get an archetype by ID.
    #[must_use]
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id.as_raw() as usize)
    }

    /// Get a mutable archetype by ID.
    #[must_use]
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id.as_raw() as usize)
    }

    /// Get the number of archetypes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.archetypes.len()
    }

    /// Check if storage is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        // Always has at least the empty archetype
        self.archetypes.len() <= 1
    }

    /// Iterate over all archetypes.
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    /// Iterate over archetypes that contain ALL of the given components.
    pub fn iter_matching(
        &self,
        required: &HashSet<ComponentId>,
    ) -> impl Iterator<Item = &Archetype> {
        self.archetypes
            .iter()
            .filter(move |arch| required.iter().all(|id| arch.contains(*id)))
    }

    /// Find the archetype ID for a given component set, if it exists.
    #[must_use]
    pub fn find(&self, component_ids: &[ComponentId]) -> Option<ArchetypeId> {
        let mut key: SmallVec<[ComponentId; 8]> = component_ids.iter().copied().collect();
        key.sort_unstable();
        self.archetype_map.get(&key).copied()
    }

    /// Get the archetype that results from adding a component to another archetype.
    pub fn with_component(
        &mut self,
        base: ArchetypeId,
        component_id: ComponentId,
        registry: &ComponentRegistry,
    ) -> ArchetypeId {
        let base_arch = &self.archetypes[base.as_raw() as usize];

        if base_arch.contains(component_id) {
            return base;
        }

        let mut new_components: SmallVec<[ComponentId; 8]> = base_arch.components.clone();
        new_components.push(component_id);
        new_components.sort_unstable();

        if let Some(&id) = self.archetype_map.get(&new_components) {
            return id;
        }

        let id = ArchetypeId::from_raw(self.archetypes.len() as u32);
        let archetype = Archetype::new(id, &new_components, registry);

        self.archetype_map.insert(new_components, id);
        self.archetypes.push(archetype);

        id
    }

    /// Get the archetype that results from removing a component from another archetype.
    pub fn without_component(
        &mut self,
        base: ArchetypeId,
        component_id: ComponentId,
        registry: &ComponentRegistry,
    ) -> ArchetypeId {
        let base_arch = &self.archetypes[base.as_raw() as usize];

        if !base_arch.contains(component_id) {
            return base;
        }

        let new_components: SmallVec<[ComponentId; 8]> = base_arch
            .components
            .iter()
            .copied()
            .filter(|&id| id != component_id)
            .collect();

        if let Some(&id) = self.archetype_map.get(&new_components) {
            return id;
        }

        let id = ArchetypeId::from_raw(self.archetypes.len() as u32);
        let archetype = Archetype::new(id, &new_components, registry);

        self.archetype_map.insert(new_components, id);
        self.archetypes.push(archetype);

        id
    }
}

impl fmt::Debug for ArchetypeStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchetypeStorage")
            .field("archetype_count", &self.archetypes.len())
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

    #[test]
    fn test_archetype_creation() {
        let mut registry = ComponentRegistry::new();
        let pos_id = registry.register::<Position>();
        let vel_id = registry.register::<Velocity>();

        let archetype = Archetype::new(ArchetypeId::from_raw(1), &[pos_id, vel_id], &registry);

        assert!(archetype.contains(pos_id));
        assert!(archetype.contains(vel_id));
        assert_eq!(archetype.components().len(), 2);
    }

    #[test]
    fn test_archetype_storage() {
        let mut registry = ComponentRegistry::new();
        let pos_id = registry.register::<Position>();
        let vel_id = registry.register::<Velocity>();

        let mut storage = ArchetypeStorage::new();

        let arch1 = storage.get_or_create(&[pos_id], &registry);
        let arch2 = storage.get_or_create(&[pos_id, vel_id], &registry);
        let arch3 = storage.get_or_create(&[pos_id], &registry);

        assert_ne!(arch1, arch2);
        assert_eq!(arch1, arch3); // Same components = same archetype
    }

    #[test]
    fn test_archetype_with_component() {
        let mut registry = ComponentRegistry::new();
        let pos_id = registry.register::<Position>();
        let vel_id = registry.register::<Velocity>();

        let mut storage = ArchetypeStorage::new();

        let pos_only = storage.get_or_create(&[pos_id], &registry);
        let pos_vel = storage.with_component(pos_only, vel_id, &registry);

        assert_ne!(pos_only, pos_vel);

        let arch = storage.get(pos_vel).unwrap();
        assert!(arch.contains(pos_id));
        assert!(arch.contains(vel_id));
    }
}
