//! Entity identifiers with generational indices.
//!
//! Entities use a generational index pattern to safely reuse IDs
//! while detecting use-after-free scenarios.

use std::fmt;

/// Generation counter to detect stale entity references.
/// Incremented each time an entity slot is recycled.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Generation(u32);

impl Generation {
    /// Create a new generation (starts at 0).
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Increment the generation counter.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Get the raw generation value.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "gen{}", self.0)
    }
}

/// Raw entity index into the entity storage.
pub type EntityId = u32;

/// A unique identifier for an entity in the world.
///
/// Entities are represented as a combination of:
/// - `id`: Index into the entity array
/// - `generation`: Version counter to detect stale references
///
/// This allows safe entity ID reuse while detecting dangling references.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    /// Index into the entity array.
    id: EntityId,
    /// Generation counter for this slot.
    generation: Generation,
}

impl Entity {
    /// The global "world" entity for storing global state.
    ///
    /// This entity is reserved for global configuration, game time, etc.
    /// In parallel RPCs, this entity is read-only.
    /// In sequential RPCs, this entity can be mutated.
    pub const WORLD: Entity = Entity {
        id: 0,
        generation: Generation(0),
    };
}

impl Entity {
    /// Create a new entity with the given ID and generation.
    #[must_use]
    pub const fn new(id: EntityId, generation: Generation) -> Self {
        Self { id, generation }
    }

    /// Get the entity's index.
    #[must_use]
    pub const fn id(self) -> EntityId {
        self.id
    }

    /// Get the entity's generation.
    #[must_use]
    pub const fn generation(self) -> Generation {
        self.generation
    }

    /// Pack entity into a single u64 for efficient storage/transmission.
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        ((self.generation.0 as u64) << 32) | (self.id as u64)
    }

    /// Unpack entity from a u64.
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self {
            id: bits as u32,
            generation: Generation((bits >> 32) as u32),
        }
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}v{})", self.id, self.generation.0)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.id, self.generation.0)
    }
}

/// Allocator for entity IDs with generation tracking.
///
/// Maintains a free list of recycled entity slots and tracks
/// the current generation for each slot.
pub struct EntityAllocator {
    /// Generation for each entity slot.
    generations: Vec<Generation>,
    /// Free list of available entity IDs.
    free_list: Vec<EntityId>,
    /// Number of currently alive entities.
    alive_count: u32,
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityAllocator {
    /// Create a new entity allocator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
            alive_count: 0,
        }
    }

    /// Create an allocator with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            generations: Vec::with_capacity(capacity),
            free_list: Vec::with_capacity(capacity / 4),
            alive_count: 0,
        }
    }

    /// Allocate a new entity.
    pub fn allocate(&mut self) -> Entity {
        self.alive_count += 1;

        if let Some(id) = self.free_list.pop() {
            // Reuse a recycled slot
            let generation = self.generations[id as usize];
            Entity::new(id, generation)
        } else {
            // Allocate a new slot
            let id = self.generations.len() as EntityId;
            let generation = Generation::new();
            self.generations.push(generation);
            Entity::new(id, generation)
        }
    }

    /// Deallocate an entity, making its slot available for reuse.
    ///
    /// Returns `true` if the entity was valid and deallocated.
    pub fn deallocate(&mut self, entity: Entity) -> bool {
        let id = entity.id() as usize;

        if id >= self.generations.len() {
            return false;
        }

        if self.generations[id] != entity.generation() {
            return false;
        }

        // Increment generation to invalidate existing references
        self.generations[id] = self.generations[id].next();
        self.free_list.push(entity.id());
        self.alive_count -= 1;
        true
    }

    /// Check if an entity is currently alive.
    #[must_use]
    pub fn is_alive(&self, entity: Entity) -> bool {
        let id = entity.id() as usize;
        id < self.generations.len() && self.generations[id] == entity.generation()
    }

    /// Get the number of currently alive entities.
    #[must_use]
    pub const fn alive_count(&self) -> u32 {
        self.alive_count
    }

    /// Get the total capacity (including recycled slots).
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.generations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_allocation() {
        let mut allocator = EntityAllocator::new();

        let e1 = allocator.allocate();
        let e2 = allocator.allocate();

        assert_eq!(e1.id(), 0);
        assert_eq!(e2.id(), 1);
        assert!(allocator.is_alive(e1));
        assert!(allocator.is_alive(e2));
        assert_eq!(allocator.alive_count(), 2);
    }

    #[test]
    fn test_entity_deallocation() {
        let mut allocator = EntityAllocator::new();

        let e1 = allocator.allocate();
        assert!(allocator.deallocate(e1));
        assert!(!allocator.is_alive(e1));
        assert_eq!(allocator.alive_count(), 0);

        // New allocation reuses the slot but with incremented generation
        let e2 = allocator.allocate();
        assert_eq!(e2.id(), e1.id());
        assert_ne!(e2.generation(), e1.generation());
    }

    #[test]
    fn test_entity_bits_roundtrip() {
        let entity = Entity::new(12345, Generation(67890));
        let bits = entity.to_bits();
        let recovered = Entity::from_bits(bits);
        assert_eq!(entity, recovered);
    }
}
