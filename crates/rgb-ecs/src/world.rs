//! World - the main container for all ECS data.
//!
//! The World holds all entities, components, and archetypes.
//! It provides the primary API for spawning entities, adding/removing
//! components, and querying data.
//!
//! Global state is stored on `Entity::WORLD` (id=0) which is automatically
//! created and marked with the `Global` component.

use std::any::TypeId;

use crate::{
    archetype::{ArchetypeId, ArchetypeStorage},
    component::{ComponentId, ComponentRegistry},
    entity::{Entity, EntityAllocator},
    relation::Pair,
};

/// Location of an entity within the archetype storage.
#[derive(Clone, Copy, Debug)]
pub struct EntityLocation {
    /// The archetype containing this entity.
    pub archetype_id: ArchetypeId,
    /// Row index within the archetype.
    pub row: usize,
}

/// Metadata for a live entity.
#[derive(Clone, Copy, Debug)]
struct EntityMeta {
    /// Current location in archetype storage.
    location: EntityLocation,
}

/// Marker component for global entities (like `Entity::WORLD`).
///
/// Global entities are read-only during parallel RPC execution
/// and can only be mutated in sequential RPCs.
#[derive(Debug, Clone, Copy, Default)]
pub struct Global;

/// The ECS world - container for all entities and components.
pub struct World {
    /// Entity ID allocator.
    entities: EntityAllocator,
    /// Entity metadata indexed by entity ID.
    entity_meta: Vec<Option<EntityMeta>>,
    /// Component type registry.
    components: ComponentRegistry,
    /// Archetype storage.
    archetypes: ArchetypeStorage,
    /// Named entity index: name bytes -> Entity
    /// This enables O(log n) lookups by name without external HashMaps.
    /// Names are arbitrary bytes (e.g., b"chunk" ++ x.to_le_bytes() ++ z.to_le_bytes()).
    name_index: std::collections::BTreeMap<Vec<u8>, Entity>,
    /// Reverse index: Entity -> name bytes (for cleanup on despawn)
    entity_names: Vec<Option<Vec<u8>>>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// Create a new world with the WORLD entity pre-allocated.
    #[must_use]
    pub fn new() -> Self {
        let mut world = Self {
            entities: EntityAllocator::new(),
            entity_meta: Vec::new(),
            components: ComponentRegistry::new(),
            archetypes: ArchetypeStorage::new(),
            name_index: std::collections::BTreeMap::new(),
            entity_names: Vec::new(),
        };

        // Reserve Entity::WORLD (id=0) and mark it as global
        let world_entity = world.spawn_empty();
        debug_assert_eq!(world_entity, Entity::WORLD);
        world.insert(Entity::WORLD, Global);

        world
    }

    /// Create a world with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(entity_capacity: usize) -> Self {
        let mut world = Self {
            entities: EntityAllocator::with_capacity(entity_capacity),
            entity_meta: Vec::with_capacity(entity_capacity),
            components: ComponentRegistry::new(),
            archetypes: ArchetypeStorage::new(),
            name_index: std::collections::BTreeMap::new(),
            entity_names: Vec::with_capacity(entity_capacity),
        };

        // Reserve Entity::WORLD (id=0) and mark it as global
        let world_entity = world.spawn_empty();
        debug_assert_eq!(world_entity, Entity::WORLD);
        world.insert(Entity::WORLD, Global);

        world
    }

    /// Check if an entity is a global entity (like `Entity::WORLD`).
    ///
    /// Global entities are read-only during parallel RPC execution.
    #[must_use]
    pub fn is_global(&self, entity: Entity) -> bool {
        self.has::<Global>(entity)
    }

    // ==================== Entity Operations ====================

    /// Spawn a new empty entity.
    pub fn spawn_empty(&mut self) -> Entity {
        let entity = self.entities.allocate();
        let id = entity.id() as usize;

        // Ensure meta vec is large enough
        if id >= self.entity_meta.len() {
            self.entity_meta.resize(id + 1, None);
        }

        // Ensure entity_names vec is large enough
        if id >= self.entity_names.len() {
            self.entity_names.resize(id + 1, None);
        }

        // Place in empty archetype
        let archetype = self.archetypes.get_mut(ArchetypeId::EMPTY).unwrap();
        let row = archetype.allocate(entity);

        self.entity_meta[id] = Some(EntityMeta {
            location: EntityLocation {
                archetype_id: ArchetypeId::EMPTY,
                row,
            },
        });

        entity
    }

    // ==================== Named Entity Operations ====================

    /// Get or create an entity by name.
    ///
    /// Names are arbitrary byte sequences, enabling efficient key encoding:
    /// - Chunks: `b"chunk" ++ x.to_le_bytes() ++ z.to_le_bytes()`
    /// - Players by UUID: `b"player" ++ uuid.to_le_bytes()`
    /// - Connections: `b"conn" ++ connection_id.to_le_bytes()`
    ///
    /// Returns the existing entity if one exists with this name, otherwise
    /// creates a new empty entity and associates it with the name.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Create a chunk key helper
    /// fn chunk_key(x: i32, z: i32) -> [u8; 13] {
    ///     let mut key = [0u8; 13];
    ///     key[0..5].copy_from_slice(b"chunk");
    ///     key[5..9].copy_from_slice(&x.to_le_bytes());
    ///     key[9..13].copy_from_slice(&z.to_le_bytes());
    ///     key
    /// }
    ///
    /// let chunk = world.entity_named(&chunk_key(10, 20));
    /// world.insert(chunk, ChunkData::new());
    /// ```
    pub fn entity_named(&mut self, name: &[u8]) -> Entity {
        // Check if entity already exists with this name
        if let Some(&entity) = self.name_index.get(name) {
            if self.is_alive(entity) {
                return entity;
            }
            // Entity was despawned but name still in index - clean up and create new
            self.name_index.remove(name);
        }

        // Create new entity and associate with name
        let entity = self.spawn_empty();
        let id = entity.id() as usize;

        self.name_index.insert(name.to_vec(), entity);
        self.entity_names[id] = Some(name.to_vec());

        entity
    }

    /// Lookup an entity by name without creating it.
    ///
    /// Returns `None` if no entity exists with this name.
    #[must_use]
    pub fn lookup(&self, name: &[u8]) -> Option<Entity> {
        let &entity = self.name_index.get(name)?;
        if self.is_alive(entity) {
            Some(entity)
        } else {
            None
        }
    }

    /// Get the name of an entity, if it has one.
    #[must_use]
    pub fn entity_name(&self, entity: Entity) -> Option<&[u8]> {
        let id = entity.id() as usize;
        self.entity_names.get(id)?.as_deref()
    }

    /// Set or update the name of an entity.
    ///
    /// If the entity already has a name, it is replaced.
    /// If another entity has this name, returns `false` and does nothing.
    pub fn set_entity_name(&mut self, entity: Entity, name: &[u8]) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        // Check if name is already taken by another entity
        if let Some(&existing) = self.name_index.get(name) {
            if existing != entity && self.is_alive(existing) {
                return false; // Name already taken
            }
        }

        let id = entity.id() as usize;

        // Remove old name if exists
        if let Some(old_name) = self.entity_names.get(id).and_then(|n| n.as_ref()) {
            self.name_index.remove(old_name);
        }

        // Ensure entity_names vec is large enough
        if id >= self.entity_names.len() {
            self.entity_names.resize(id + 1, None);
        }

        // Set new name
        self.name_index.insert(name.to_vec(), entity);
        self.entity_names[id] = Some(name.to_vec());

        true
    }

    /// Spawn an entity with a single component.
    pub fn spawn<T: 'static + Send + Sync>(&mut self, component: T) -> Entity {
        let entity = self.entities.allocate();
        let id = entity.id() as usize;

        // Ensure meta vec is large enough
        if id >= self.entity_meta.len() {
            self.entity_meta.resize(id + 1, None);
        }

        // Ensure entity_names vec is large enough
        if id >= self.entity_names.len() {
            self.entity_names.resize(id + 1, None);
        }

        // Get or create archetype for this component
        let comp_id = self.components.register::<T>();
        let arch_id = self.archetypes.get_or_create(&[comp_id], &self.components);

        let archetype = self.archetypes.get_mut(arch_id).unwrap();
        let row = archetype.allocate(entity);

        // SAFETY: We just allocated this row and T matches comp_id
        unsafe {
            archetype.set_component(comp_id, row, component);
        }

        self.entity_meta[id] = Some(EntityMeta {
            location: EntityLocation {
                archetype_id: arch_id,
                row,
            },
        });

        entity
    }

    /// Despawn an entity, removing it and all its components.
    ///
    /// Returns `true` if the entity existed and was removed.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let id = entity.id() as usize;
        let meta = match self.entity_meta.get(id).and_then(|m| *m) {
            Some(m) => m,
            None => return false,
        };

        let archetype = self.archetypes.get_mut(meta.location.archetype_id).unwrap();
        let num_columns = archetype.components().len();

        // Remove component data from all columns
        for col_idx in 0..num_columns {
            let column = archetype.column_by_index_mut(col_idx).unwrap();
            // SAFETY: meta.location.row is valid
            unsafe {
                column.swap_remove_drop(meta.location.row);
            }
        }

        // Remove entity from archetype
        // SAFETY: We just removed all component data
        let swapped = unsafe { archetype.deallocate(meta.location.row) };

        // Update swapped entity's metadata if needed
        if let Some(swapped_entity) = swapped {
            let swapped_id = swapped_entity.id() as usize;
            if let Some(Some(swapped_meta)) = self.entity_meta.get_mut(swapped_id) {
                swapped_meta.location.row = meta.location.row;
            }
        }

        // Clean up name index
        if let Some(Some(name)) = self.entity_names.get(id) {
            self.name_index.remove(name);
        }
        if id < self.entity_names.len() {
            self.entity_names[id] = None;
        }

        // Clear our metadata
        self.entity_meta[id] = None;

        // Free the entity ID
        self.entities.deallocate(entity);

        true
    }

    /// Check if an entity is alive.
    #[must_use]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    /// Get the number of alive entities.
    #[must_use]
    pub fn entity_count(&self) -> u32 {
        self.entities.alive_count()
    }

    /// Get the location of an entity.
    #[must_use]
    pub fn entity_location(&self, entity: Entity) -> Option<EntityLocation> {
        if !self.entities.is_alive(entity) {
            return None;
        }
        self.entity_meta
            .get(entity.id() as usize)
            .and_then(|m| *m)
            .map(|m| m.location)
    }

    // ==================== Component Operations ====================

    /// Register a component type.
    pub fn register_component<T: 'static + Send + Sync>(&mut self) -> ComponentId {
        self.components.register::<T>()
    }

    /// Get the component ID for a type.
    #[must_use]
    pub fn component_id<T: 'static + Send + Sync>(&self) -> Option<ComponentId> {
        self.components.get_id::<T>()
    }

    /// Add a component to an entity.
    ///
    /// If the entity already has this component type, it is replaced.
    pub fn insert<T: 'static + Send + Sync>(&mut self, entity: Entity, component: T) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let entity_id = entity.id() as usize;
        let meta = match self.entity_meta.get(entity_id).and_then(|m| *m) {
            Some(m) => m,
            None => return false,
        };

        let comp_id = self.components.register::<T>();

        // Check if already in correct archetype
        let old_archetype = self.archetypes.get(meta.location.archetype_id).unwrap();
        if old_archetype.contains(comp_id) {
            // Just update the existing component
            let archetype = self.archetypes.get_mut(meta.location.archetype_id).unwrap();
            // SAFETY: Entity is in this archetype and T matches comp_id
            unsafe {
                archetype.set_component(comp_id, meta.location.row, component);
            }
            return true;
        }

        // Need to move to new archetype
        let new_arch_id =
            self.archetypes
                .with_component(meta.location.archetype_id, comp_id, &self.components);

        self.move_entity_to_archetype(entity, new_arch_id, Some((comp_id, component)))
    }

    /// Remove a component from an entity.
    ///
    /// Returns the removed component if it existed.
    pub fn remove<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let comp_id = self.components.get_id::<T>()?;

        let entity_id = entity.id() as usize;
        let meta = self.entity_meta.get(entity_id).and_then(|m| *m)?;

        // Check and extract value atomically
        let value = {
            let old_archetype = self.archetypes.get(meta.location.archetype_id)?;
            if !old_archetype.contains(comp_id) {
                return None;
            }

            let column = old_archetype.column(comp_id)?;
            // SAFETY: Entity is in this archetype and T matches
            unsafe { std::ptr::read(column.get_unchecked_raw(meta.location.row).cast::<T>()) }
        };

        // Get new archetype without this component
        let new_arch_id = self.archetypes.without_component(
            meta.location.archetype_id,
            comp_id,
            &self.components,
        );

        self.move_entity_to_archetype::<()>(entity, new_arch_id, None);

        Some(value)
    }

    /// Get an owned copy of an entity's component.
    ///
    /// Returns `None` if the entity doesn't exist or doesn't have the component.
    /// This follows the SpacetimeDB pattern: get → modify → update.
    #[must_use]
    pub fn get<T: 'static + Send + Sync + Clone>(&self, entity: Entity) -> Option<T> {
        let comp_id = self.components.get_id::<T>()?;
        let meta = self.entity_meta.get(entity.id() as usize)?.as_ref()?;
        let archetype = self.archetypes.get(meta.location.archetype_id)?;

        // SAFETY: We verified the entity is alive and in this archetype
        let component_ref: &T = unsafe { archetype.get_component(comp_id, meta.location.row)? };
        Some(component_ref.clone())
    }

    /// Get a reference to an entity's component (for internal use or when Clone is expensive).
    ///
    /// Prefer `get()` for the owned-value API.
    #[must_use]
    pub fn get_ref<T: 'static + Send + Sync>(&self, entity: Entity) -> Option<&T> {
        let comp_id = self.components.get_id::<T>()?;
        let meta = self.entity_meta.get(entity.id() as usize)?.as_ref()?;
        let archetype = self.archetypes.get(meta.location.archetype_id)?;

        // SAFETY: We verified the entity is alive and in this archetype
        unsafe { archetype.get_component(comp_id, meta.location.row) }
    }

    /// Get a raw pointer to a component by TypeId.
    ///
    /// This is used by the event system to pass event data to observers.
    /// The caller must ensure the pointer is used correctly.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid while the entity exists and
    /// the component is not removed. The caller must cast to the correct type.
    #[must_use]
    pub fn get_raw_ptr(&self, entity: Entity, type_id: TypeId) -> Option<*const u8> {
        let comp_id = self.components.get_id_by_type_id(type_id)?;
        let meta = self.entity_meta.get(entity.id() as usize)?.as_ref()?;
        let archetype = self.archetypes.get(meta.location.archetype_id)?;

        // Check if archetype has this component
        let col_idx = archetype.column_index(comp_id)?;

        // Get raw pointer to the component data
        Some(archetype.column_ptr(col_idx, meta.location.row))
    }

    /// Update an entity's component with a new value.
    ///
    /// This is the write-back part of the get → modify → update pattern.
    /// Returns `false` if the entity doesn't exist or doesn't have the component.
    pub fn update<T: 'static + Send + Sync>(&mut self, entity: Entity, component: T) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let Some(comp_id) = self.components.get_id::<T>() else {
            return false;
        };

        let entity_id = entity.id() as usize;
        let Some(Some(meta)) = self.entity_meta.get(entity_id) else {
            return false;
        };

        let Some(archetype) = self.archetypes.get_mut(meta.location.archetype_id) else {
            return false;
        };

        if !archetype.contains(comp_id) {
            return false;
        }

        // SAFETY: Entity is in this archetype and T matches comp_id
        unsafe {
            archetype.set_component(comp_id, meta.location.row, component);
        }
        true
    }

    /// Check if an entity has a component.
    #[must_use]
    pub fn has<T: 'static + Send + Sync>(&self, entity: Entity) -> bool {
        let Some(comp_id) = self.components.get_id::<T>() else {
            return false;
        };
        let Some(Some(meta)) = self.entity_meta.get(entity.id() as usize) else {
            return false;
        };
        let Some(archetype) = self.archetypes.get(meta.location.archetype_id) else {
            return false;
        };
        archetype.contains(comp_id)
    }

    /// Move an entity to a new archetype, optionally adding a new component.
    fn move_entity_to_archetype<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
        new_arch_id: ArchetypeId,
        new_component: Option<(ComponentId, T)>,
    ) -> bool {
        let entity_id = entity.id() as usize;
        let meta = match self.entity_meta.get(entity_id).and_then(|m| *m) {
            Some(m) => m,
            None => return false,
        };

        if meta.location.archetype_id == new_arch_id {
            return true;
        }

        // Get both archetypes - need to use raw pointers for simultaneous mutable access
        let archetypes_ptr =
            self.archetypes.get_mut(ArchetypeId::EMPTY).unwrap() as *mut _ as *mut u8;

        let old_arch_id = meta.location.archetype_id;
        let old_row = meta.location.row;

        // SAFETY: We're getting different archetypes
        let (old_arch, new_arch) = unsafe {
            let base = archetypes_ptr as *mut crate::archetype::Archetype;
            (
                &mut *base.add(old_arch_id.as_raw() as usize),
                &mut *base.add(new_arch_id.as_raw() as usize),
            )
        };

        // Allocate space in new archetype
        let new_row = new_arch.allocate(entity);

        // Collect component indices first to avoid borrow issues
        let components_to_copy: Vec<(ComponentId, usize, usize)> = old_arch
            .components()
            .iter()
            .filter_map(|&comp_id| {
                let old_col_idx = old_arch.column_index(comp_id)?;
                let new_col_idx = new_arch.column_index(comp_id)?;
                Some((comp_id, old_col_idx, new_col_idx))
            })
            .collect();

        // Copy existing components to new archetype
        for (comp_id, old_col_idx, new_col_idx) in components_to_copy {
            let info = self.components.get_info(comp_id).unwrap();
            let _size = info.size();

            // Get raw pointers
            let old_col = old_arch.column_by_index_mut(old_col_idx).unwrap();
            let old_ptr = unsafe { old_col.get_unchecked_raw(old_row) };

            let new_col = new_arch.column_by_index_mut(new_col_idx).unwrap();

            // SAFETY: We're copying valid component data
            unsafe {
                new_col.push_raw(old_ptr);
            }
        }

        // Add new component if provided
        if let Some((comp_id, component)) = new_component {
            // SAFETY: new_row is valid and T matches comp_id
            unsafe {
                new_arch.set_component(comp_id, new_row, component);
            }
        }

        // Remove from old archetype (don't drop, we moved the data)
        for col_idx in 0..old_arch.components().len() {
            let column = old_arch.column_by_index_mut(col_idx).unwrap();
            // Use a dummy buffer - we don't need the data
            let mut dummy = [0u8; 256];
            // SAFETY: old_row is valid
            unsafe {
                column.swap_remove_raw(old_row, dummy.as_mut_ptr());
            }
        }

        // SAFETY: We removed all component data
        let swapped = unsafe { old_arch.deallocate(old_row) };

        // Update swapped entity's metadata
        if let Some(swapped_entity) = swapped {
            let swapped_id = swapped_entity.id() as usize;
            if let Some(Some(swapped_meta)) = self.entity_meta.get_mut(swapped_id) {
                swapped_meta.location.row = old_row;
            }
        }

        // Update our metadata
        self.entity_meta[entity_id] = Some(EntityMeta {
            location: EntityLocation {
                archetype_id: new_arch_id,
                row: new_row,
            },
        });

        true
    }

    // ==================== Relation/Pair Operations ====================

    /// Add a relation pair to an entity.
    ///
    /// A pair `(R, target)` represents a relationship from this entity to `target`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let parent = world.spawn_empty();
    /// let child = world.spawn_empty();
    /// world.insert_pair::<ChildOf>(child, parent);
    /// ```
    pub fn insert_pair<R: 'static + Send + Sync + Default>(
        &mut self,
        entity: Entity,
        target: Entity,
    ) -> bool {
        // Store the pair as a component: Pair<R> where R is the relation type
        self.insert(entity, Pair::<R>::new(target))
    }

    /// Get the target of a relation pair.
    ///
    /// Returns the target entity if this entity has the relation.
    #[must_use]
    pub fn get_pair_target<R: 'static + Send + Sync + Clone>(
        &self,
        entity: Entity,
    ) -> Option<Entity> {
        let pair: Pair<R> = self.get(entity)?;
        Some(pair.target())
    }

    /// Check if an entity has a specific relation pair.
    #[must_use]
    pub fn has_pair<R: 'static + Send + Sync + Clone>(
        &self,
        entity: Entity,
        target: Entity,
    ) -> bool {
        self.get_pair_target::<R>(entity)
            .is_some_and(|t| t == target)
    }

    /// Check if an entity has any pair of the given relation type.
    #[must_use]
    pub fn has_relation<R: 'static + Send + Sync>(&self, entity: Entity) -> bool {
        self.has::<Pair<R>>(entity)
    }

    /// Remove a relation pair from an entity.
    ///
    /// Returns the target entity if the relation existed.
    pub fn remove_pair<R: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<Entity> {
        let pair: Pair<R> = self.remove(entity)?;
        Some(pair.target())
    }

    /// Get the parent of an entity (via ChildOf relation).
    #[must_use]
    pub fn parent(&self, entity: Entity) -> Option<Entity> {
        self.get_pair_target::<crate::relation::ChildOf>(entity)
    }

    /// Set the parent of an entity (via ChildOf relation).
    pub fn set_parent(&mut self, child: Entity, parent: Entity) -> bool {
        self.insert_pair::<crate::relation::ChildOf>(child, parent)
    }

    // ==================== Archetype Access ====================

    /// Get the component registry.
    #[must_use]
    pub fn components(&self) -> &ComponentRegistry {
        &self.components
    }

    /// Get the archetype storage.
    #[must_use]
    pub fn archetypes(&self) -> &ArchetypeStorage {
        &self.archetypes
    }

    /// Get mutable archetype storage.
    #[must_use]
    pub fn archetypes_mut(&mut self) -> &mut ArchetypeStorage {
        &mut self.archetypes
    }

    // ==================== Query ====================

    /// Iterate over all entities that have a specific component.
    ///
    /// Returns an iterator of `(Entity, T)` pairs.
    pub fn query<T: 'static + Send + Sync + Clone>(
        &self,
    ) -> impl Iterator<Item = (Entity, T)> + '_ {
        let comp_id = match self.components.get_id::<T>() {
            Some(id) => id,
            None => return QueryIter::empty(),
        };

        let mut required = hashbrown::HashSet::new();
        required.insert(comp_id);

        QueryIter::new(self, required)
    }

    /// Iterate over all entities (no component filter).
    pub fn entities_iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.archetypes
            .iter()
            .flat_map(|arch| arch.entities().iter().copied())
    }
}

/// Iterator for query results.
pub struct QueryIter<'w, T> {
    world: Option<&'w World>,
    archetype_iter: Box<dyn Iterator<Item = &'w crate::archetype::Archetype> + 'w>,
    current_entities: std::slice::Iter<'w, Entity>,
    _marker: std::marker::PhantomData<T>,
}

impl<'w, T: 'static + Send + Sync + Clone> QueryIter<'w, T> {
    fn new(world: &'w World, required: hashbrown::HashSet<ComponentId>) -> Self {
        let archetype_iter = Box::new(
            world
                .archetypes
                .iter()
                .filter(move |arch| required.iter().all(|id| arch.contains(*id))),
        );
        Self {
            world: Some(world),
            archetype_iter,
            current_entities: [].iter(),
            _marker: std::marker::PhantomData,
        }
    }

    fn empty() -> Self {
        Self {
            world: None,
            archetype_iter: Box::new(std::iter::empty()),
            current_entities: [].iter(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'w, T: 'static + Send + Sync + Clone> Iterator for QueryIter<'w, T> {
    type Item = (Entity, T);

    fn next(&mut self) -> Option<Self::Item> {
        let world = self.world?;

        loop {
            // Try to get next entity from current archetype
            if let Some(&entity) = self.current_entities.next() {
                if let Some(value) = world.get::<T>(entity) {
                    return Some((entity, value));
                }
                continue;
            }

            // Move to next archetype
            let arch = self.archetype_iter.next()?;
            self.current_entities = arch.entities().iter();
        }
    }
}

/// A plugin that can be added to a World to register components, systems, and initial state.
///
/// This is the primary way to modularize ECS code across crates.
///
/// # Example
///
/// ```ignore
/// struct PhysicsPlugin;
///
/// impl Plugin for PhysicsPlugin {
///     fn build(&self, world: &mut World) {
///         // Register components
///         world.register::<Velocity>();
///         world.register::<Acceleration>();
///
///         // Initialize global state
///         world.insert(Entity::WORLD, PhysicsConfig::default());
///     }
/// }
///
/// // Usage
/// let mut world = World::new();
/// world.add_plugin(PhysicsPlugin);
/// ```
pub trait Plugin {
    /// Build/configure the world with this plugin's components and state.
    fn build(&self, world: &mut World);
}

impl World {
    /// Add a plugin to this world.
    ///
    /// Plugins are a way to modularize ECS setup code.
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    /// Register a component type without creating any entities.
    ///
    /// This is useful for plugins that want to ensure component types
    /// are registered before any systems run.
    pub fn register<T: 'static + Send + Sync>(&mut self) -> ComponentId {
        self.components.register::<T>()
    }

    /// Register a transient component type (will not be persisted).
    ///
    /// Use this for network buffers, caches, runtime handles, and other
    /// components that should not be saved to storage.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // At startup, register transient components
    /// world.register_transient::<PacketBuffer>();
    /// world.register_transient::<NetworkIngress>();
    /// ```
    pub fn register_transient<T: 'static + Send + Sync>(&mut self) -> ComponentId {
        self.components.register_transient::<T>()
    }

    /// Check if a component type is transient (will not be persisted).
    #[must_use]
    pub fn is_transient<T: 'static + Send + Sync>(&self) -> bool {
        self.components.is_transient::<T>()
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("entity_count", &self.entities.alive_count())
            .field("component_types", &self.components.len())
            .field("archetype_count", &self.archetypes.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Health(u32);

    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    struct GameTime {
        tick: u64,
    }

    #[test]
    fn test_spawn_and_get() {
        let mut world = World::new();

        let entity = world.spawn(Position { x: 1.0, y: 2.0 });

        assert!(world.is_alive(entity));
        // World starts with Entity::WORLD, so spawning one more = 2
        assert_eq!(world.entity_count(), 2);

        // get() now returns owned value
        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn test_get_modify_update() {
        let mut world = World::new();

        let entity = world.spawn(Position { x: 1.0, y: 2.0 });

        // Get owned value
        let mut pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);

        // Modify locally
        pos.x += 10.0;

        // Write back
        assert!(world.update(entity, pos));

        // Verify update
        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 11.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn test_despawn() {
        let mut world = World::new();

        let entity = world.spawn(Position { x: 1.0, y: 2.0 });
        assert!(world.is_alive(entity));

        assert!(world.despawn(entity));
        assert!(!world.is_alive(entity));
        // Entity::WORLD still exists
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_insert_component() {
        let mut world = World::new();

        let entity = world.spawn(Position { x: 1.0, y: 2.0 });

        assert!(world.insert(entity, Velocity { x: 0.5, y: 0.5 }));

        assert!(world.has::<Position>(entity));
        assert!(world.has::<Velocity>(entity));

        // get() returns owned value
        let vel = world.get::<Velocity>(entity).unwrap();
        assert_eq!(vel.x, 0.5);
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();

        let entity = world.spawn(Position { x: 1.0, y: 2.0 });
        world.insert(entity, Velocity { x: 0.5, y: 0.5 });

        let removed = world.remove::<Velocity>(entity);
        assert_eq!(removed, Some(Velocity { x: 0.5, y: 0.5 }));

        assert!(world.has::<Position>(entity));
        assert!(!world.has::<Velocity>(entity));
    }

    #[test]
    fn test_global_entity() {
        let mut world = World::new();

        // Entity::WORLD exists by default and is marked as Global
        assert!(world.is_alive(Entity::WORLD));
        assert!(world.is_global(Entity::WORLD));

        // Store global state on Entity::WORLD using owned values
        world.insert(Entity::WORLD, GameTime { tick: 0 });

        // Read (get returns owned value)
        let time = world.get::<GameTime>(Entity::WORLD).unwrap();
        assert_eq!(time.tick, 0);

        // Update using owned value pattern
        let mut time = world.get::<GameTime>(Entity::WORLD).unwrap();
        time.tick = 100;
        world.update(Entity::WORLD, time);

        // Verify update
        let time = world.get::<GameTime>(Entity::WORLD).unwrap();
        assert_eq!(time.tick, 100);
    }

    #[test]
    fn test_multiple_entities() {
        let mut world = World::new();

        let e1 = world.spawn(Position { x: 0.0, y: 0.0 });
        let e2 = world.spawn(Position { x: 1.0, y: 1.0 });
        let e3 = world.spawn(Position { x: 2.0, y: 2.0 });

        world.insert(e1, Velocity { x: 1.0, y: 0.0 });
        world.insert(e2, Velocity { x: 0.0, y: 1.0 });
        // e3 has no velocity

        assert!(world.has::<Velocity>(e1));
        assert!(world.has::<Velocity>(e2));
        assert!(!world.has::<Velocity>(e3));

        // 3 spawned entities + Entity::WORLD = 4
        assert_eq!(world.entity_count(), 4);
    }

    #[test]
    fn test_despawn_maintains_others() {
        let mut world = World::new();

        let e1 = world.spawn(Position { x: 1.0, y: 1.0 });
        let e2 = world.spawn(Position { x: 2.0, y: 2.0 });
        let e3 = world.spawn(Position { x: 3.0, y: 3.0 });

        world.despawn(e2);

        assert!(world.is_alive(e1));
        assert!(!world.is_alive(e2));
        assert!(world.is_alive(e3));

        // Remaining entities should still have correct data (owned values)
        assert_eq!(world.get::<Position>(e1).unwrap().x, 1.0);
        assert_eq!(world.get::<Position>(e3).unwrap().x, 3.0);
    }

    #[test]
    fn test_update_nonexistent_fails() {
        let mut world = World::new();

        let entity = world.spawn(Position { x: 1.0, y: 2.0 });

        // Update non-existent component type should fail
        assert!(!world.update(entity, Health(100)));

        // Update after despawn should fail
        world.despawn(entity);
        assert!(!world.update(entity, Position { x: 5.0, y: 5.0 }));
    }

    #[test]
    fn test_parent_child_relation() {
        use crate::relation::ChildOf;

        let mut world = World::new();

        let parent = world.spawn_empty();
        let child = world.spawn_empty();

        // Set up parent-child relationship
        assert!(world.set_parent(child, parent));

        // Check relationship exists
        assert!(world.has_relation::<ChildOf>(child));
        assert!(world.has_pair::<ChildOf>(child, parent));
        assert_eq!(world.parent(child), Some(parent));

        // Parent shouldn't have ChildOf (it's the parent, not child)
        assert!(!world.has_relation::<ChildOf>(parent));
    }

    #[test]
    fn test_relation_pairs() {
        use crate::relation::{ContainedIn, OwnedBy};

        let mut world = World::new();

        let player = world.spawn_empty();
        let inventory = world.spawn_empty();
        let sword = world.spawn_empty();

        // Inventory owned by player
        world.insert_pair::<OwnedBy>(inventory, player);

        // Sword contained in inventory
        world.insert_pair::<ContainedIn>(sword, inventory);

        // Check relationships
        assert_eq!(world.get_pair_target::<OwnedBy>(inventory), Some(player));
        assert_eq!(world.get_pair_target::<ContainedIn>(sword), Some(inventory));

        // Remove relationship
        let removed = world.remove_pair::<ContainedIn>(sword);
        assert_eq!(removed, Some(inventory));
        assert!(!world.has_relation::<ContainedIn>(sword));
    }

    #[test]
    fn test_named_entity_basic() {
        let mut world = World::new();

        // Create named entity
        let e1 = world.entity_named(b"test_entity");
        assert!(world.is_alive(e1));

        // Getting same name returns same entity
        let e2 = world.entity_named(b"test_entity");
        assert_eq!(e1, e2);

        // Lookup works
        assert_eq!(world.lookup(b"test_entity"), Some(e1));
        assert_eq!(world.lookup(b"nonexistent"), None);

        // Can get entity name
        assert_eq!(world.entity_name(e1), Some(b"test_entity".as_slice()));
    }

    fn chunk_key(x: i32, z: i32) -> [u8; 13] {
        let mut key = [0u8; 13];
        key[0..5].copy_from_slice(b"chunk");
        key[5..9].copy_from_slice(&x.to_le_bytes());
        key[9..13].copy_from_slice(&z.to_le_bytes());
        key
    }

    #[test]
    fn test_named_entity_chunk_key() {
        let mut world = World::new();

        // Create chunks
        let chunk1 = world.entity_named(&chunk_key(10, 20));
        let chunk2 = world.entity_named(&chunk_key(-5, 100));

        world.insert(chunk1, Position { x: 10.0, y: 0.0 });
        world.insert(chunk2, Position { x: -5.0, y: 0.0 });

        // Lookup by coordinates
        let found1 = world.lookup(&chunk_key(10, 20)).unwrap();
        let found2 = world.lookup(&chunk_key(-5, 100)).unwrap();

        assert_eq!(found1, chunk1);
        assert_eq!(found2, chunk2);
        assert_eq!(world.get::<Position>(found1).unwrap().x, 10.0);
        assert_eq!(world.get::<Position>(found2).unwrap().x, -5.0);

        // Non-existent chunk
        assert!(world.lookup(&chunk_key(999, 999)).is_none());
    }

    #[test]
    fn test_named_entity_despawn_cleanup() {
        let mut world = World::new();

        let entity = world.entity_named(b"temporary");
        assert!(world.lookup(b"temporary").is_some());

        // Despawn should clean up name index
        world.despawn(entity);
        assert!(world.lookup(b"temporary").is_none());

        // Can create new entity with same name
        let new_entity = world.entity_named(b"temporary");
        assert!(world.is_alive(new_entity));
        assert_ne!(entity, new_entity); // Different entity (or at least different generation)
    }

    #[test]
    fn test_set_entity_name() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        assert!(world.entity_name(entity).is_none());

        // Set name
        assert!(world.set_entity_name(entity, b"my_entity"));
        assert_eq!(world.entity_name(entity), Some(b"my_entity".as_slice()));
        assert_eq!(world.lookup(b"my_entity"), Some(entity));

        // Rename
        assert!(world.set_entity_name(entity, b"renamed"));
        assert_eq!(world.entity_name(entity), Some(b"renamed".as_slice()));
        assert!(world.lookup(b"my_entity").is_none());
        assert_eq!(world.lookup(b"renamed"), Some(entity));
    }

    #[test]
    fn test_named_entity_uniqueness() {
        let mut world = World::new();

        let e1 = world.entity_named(b"unique");
        let e2 = world.spawn_empty();

        // Can't give e2 the same name
        assert!(!world.set_entity_name(e2, b"unique"));
        assert!(world.entity_name(e2).is_none());

        // e1 still has the name
        assert_eq!(world.lookup(b"unique"), Some(e1));
    }

    // ==================== Connection Key Tests ====================

    fn conn_key(id: u64) -> [u8; 12] {
        let mut key = [0u8; 12];
        key[0..4].copy_from_slice(b"conn");
        key[4..12].copy_from_slice(&id.to_le_bytes());
        key
    }

    #[test]
    fn test_named_entity_connection_keys() {
        let mut world = World::new();

        // Simulate connection entities
        let conn1 = world.entity_named(&conn_key(12345));
        let conn2 = world.entity_named(&conn_key(67890));
        let conn3 = world.entity_named(&conn_key(u64::MAX));

        // All different entities
        assert_ne!(conn1, conn2);
        assert_ne!(conn2, conn3);

        // Lookup works for all
        assert_eq!(world.lookup(&conn_key(12345)), Some(conn1));
        assert_eq!(world.lookup(&conn_key(67890)), Some(conn2));
        assert_eq!(world.lookup(&conn_key(u64::MAX)), Some(conn3));

        // Disconnect (despawn) cleans up
        world.despawn(conn2);
        assert!(world.lookup(&conn_key(67890)).is_none());
        assert!(world.lookup(&conn_key(12345)).is_some()); // Others unaffected
    }

    #[test]
    fn test_named_entity_binary_keys() {
        let mut world = World::new();

        // Test with various binary patterns including null bytes
        let key1 = [0u8, 1, 2, 3, 0, 0, 255, 128];
        let key2 = [0u8, 0, 0, 0, 0, 0, 0, 0]; // All zeros
        let key3 = [255u8; 8]; // All 0xFF

        let e1 = world.entity_named(&key1);
        let e2 = world.entity_named(&key2);
        let e3 = world.entity_named(&key3);

        assert_ne!(e1, e2);
        assert_ne!(e2, e3);

        assert_eq!(world.lookup(&key1), Some(e1));
        assert_eq!(world.lookup(&key2), Some(e2));
        assert_eq!(world.lookup(&key3), Some(e3));
    }

    #[test]
    fn test_named_entity_with_components() {
        let mut world = World::new();

        // Create named entity and add components
        let player = world.entity_named(b"player:uuid:12345");
        world.insert(player, Position { x: 100.0, y: 64.0 });
        world.insert(player, Health(100));

        // Lookup and verify components
        let found = world.lookup(b"player:uuid:12345").unwrap();
        assert_eq!(world.get::<Position>(found).unwrap().x, 100.0);
        assert_eq!(world.get::<Health>(found).unwrap().0, 100);

        // Update via lookup
        let mut pos = world.get::<Position>(found).unwrap();
        pos.x = 200.0;
        world.update(found, pos);

        // Re-lookup and verify update
        let found2 = world.lookup(b"player:uuid:12345").unwrap();
        assert_eq!(found, found2);
        assert_eq!(world.get::<Position>(found2).unwrap().x, 200.0);
    }

    // ==================== Transient Component Tests ====================

    #[test]
    fn test_transient_component_registration() {
        let mut world = World::new();

        // Register normal and transient components
        world.register::<Position>();
        world.register_transient::<Velocity>();

        // Check transient status
        assert!(!world.is_transient::<Position>());
        assert!(world.is_transient::<Velocity>());

        // Unregistered types are not transient
        assert!(!world.is_transient::<Health>());
    }

    #[test]
    fn test_transient_components_work_normally() {
        let mut world = World::new();

        // Register as transient
        world.register_transient::<Velocity>();

        // Still works as a normal component
        let entity = world.spawn(Velocity { x: 1.0, y: 2.0 });
        let vel = world.get::<Velocity>(entity).unwrap();
        assert_eq!(vel.x, 1.0);
        assert_eq!(vel.y, 2.0);

        // Can update
        world.update(entity, Velocity { x: 3.0, y: 4.0 });
        let vel = world.get::<Velocity>(entity).unwrap();
        assert_eq!(vel.x, 3.0);

        // Can remove
        let removed = world.remove::<Velocity>(entity);
        assert!(removed.is_some());
        assert!(!world.has::<Velocity>(entity));
    }

    #[test]
    fn test_mixed_transient_and_persistent_components() {
        let mut world = World::new();

        // Position is persistent, Velocity is transient
        world.register::<Position>();
        world.register_transient::<Velocity>();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });
        world.insert(entity, Velocity { x: 3.0, y: 4.0 });

        // Both work
        assert!(world.has::<Position>(entity));
        assert!(world.has::<Velocity>(entity));

        // But only Position should be persisted (checked via is_transient)
        assert!(!world.is_transient::<Position>());
        assert!(world.is_transient::<Velocity>());
    }

    // ==================== Stress Tests ====================

    #[test]
    fn test_many_named_entities() {
        let mut world = World::new();

        // Create 1000 named entities
        for i in 0..1000u32 {
            let key = format!("entity_{}", i);
            let entity = world.entity_named(key.as_bytes());
            world.insert(
                entity,
                Position {
                    x: i as f32,
                    y: 0.0,
                },
            );
        }

        // Verify all can be looked up
        for i in 0..1000u32 {
            let key = format!("entity_{}", i);
            let entity = world.lookup(key.as_bytes()).unwrap();
            assert_eq!(world.get::<Position>(entity).unwrap().x, i as f32);
        }

        // Despawn half
        for i in (0..1000u32).step_by(2) {
            let key = format!("entity_{}", i);
            let entity = world.lookup(key.as_bytes()).unwrap();
            world.despawn(entity);
        }

        // Verify correct entities remain
        for i in 0..1000u32 {
            let key = format!("entity_{}", i);
            if i % 2 == 0 {
                assert!(world.lookup(key.as_bytes()).is_none());
            } else {
                assert!(world.lookup(key.as_bytes()).is_some());
            }
        }
    }

    #[test]
    fn test_chunk_grid_simulation() {
        let mut world = World::new();

        // Simulate a 32x32 chunk grid
        for x in -16..16 {
            for z in -16..16 {
                let chunk = world.entity_named(&chunk_key(x, z));
                world.insert(
                    chunk,
                    Position {
                        x: x as f32 * 16.0,
                        y: z as f32 * 16.0,
                    },
                );
            }
        }

        // Total: 32 * 32 = 1024 chunks + Entity::WORLD = 1025
        assert_eq!(world.entity_count(), 1025);

        // Random access pattern (simulating player movement)
        let test_coords = [(0, 0), (-16, -16), (15, 15), (-1, 5), (8, -8)];

        for (x, z) in test_coords {
            let chunk = world.lookup(&chunk_key(x, z)).unwrap();
            let pos = world.get::<Position>(chunk).unwrap();
            assert_eq!(pos.x, x as f32 * 16.0);
            assert_eq!(pos.y, z as f32 * 16.0);
        }
    }
}
