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

    /// Spawn an entity with a single component.
    pub fn spawn<T: 'static + Send + Sync>(&mut self, component: T) -> Entity {
        let entity = self.entities.allocate();
        let id = entity.id() as usize;

        // Ensure meta vec is large enough
        if id >= self.entity_meta.len() {
            self.entity_meta.resize(id + 1, None);
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
}
