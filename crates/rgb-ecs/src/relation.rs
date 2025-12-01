//! Relations and Pairs - Flecs-inspired entity relationships.
//!
//! A pair `(Relation, Target)` represents a relationship between entities.
//! For example:
//! - `(ChildOf, parent)` - entity is a child of parent
//! - `(ContainedIn, inventory)` - item is contained in inventory
//! - `(Requires, fuel)` - entity requires fuel
//!
//! Pairs can be used as component identifiers, enabling queries like:
//! ```ignore
//! // Query all entities with any ChildOf relation
//! world.query::<(ChildOf, _)>();
//!
//! // Query all children of a specific parent
//! world.query::<(ChildOf, parent_entity)>();
//! ```

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::entity::Entity;

/// A pair combines a relation type with a target entity.
///
/// Pairs are used to express relationships between entities.
/// The relation is typically a marker type (like `ChildOf`), and
/// the target is an entity ID.
///
/// # Example
///
/// ```ignore
/// // Create a ChildOf relation
/// struct ChildOf;
///
/// let parent = world.spawn_empty();
/// let child = world.spawn_empty();
///
/// // Add the relation
/// world.insert_pair::<ChildOf>(child, parent);
///
/// // Check the relation
/// assert!(world.has_pair::<ChildOf>(child, parent));
/// ```
#[derive(Clone, Copy)]
pub struct Pair<R> {
    /// The target entity of the relation
    pub target: Entity,
    /// Phantom data for the relation type
    _marker: PhantomData<R>,
}

impl<R> Pair<R> {
    /// Create a new pair with the given target.
    #[must_use]
    pub const fn new(target: Entity) -> Self {
        Self {
            target,
            _marker: PhantomData,
        }
    }

    /// Get the target entity.
    #[must_use]
    pub const fn target(&self) -> Entity {
        self.target
    }
}

impl<R> PartialEq for Pair<R> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
    }
}

impl<R> Eq for Pair<R> {}

impl<R> Hash for Pair<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
    }
}

impl<R> fmt::Debug for Pair<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pair<{}>({:?})", std::any::type_name::<R>(), self.target)
    }
}

// ============================================================================
// Built-in relation types
// ============================================================================

/// Parent-child relationship.
///
/// `(ChildOf, parent)` means "this entity is a child of parent".
/// This is the fundamental hierarchical relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChildOf;

/// Ownership relationship.
///
/// `(OwnedBy, owner)` means "this entity is owned by owner".
/// Useful for items, inventories, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct OwnedBy;

/// Containment relationship.
///
/// `(ContainedIn, container)` means "this entity is contained in container".
/// Useful for inventory systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ContainedIn;

/// Dependency relationship.
///
/// `(Requires, dependency)` means "this entity requires dependency".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Requires;

/// Instance-of relationship (prefab/archetype pattern).
///
/// `(InstanceOf, prefab)` means "this entity is an instance of prefab".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct InstanceOf;

// ============================================================================
// Pair ID - packed representation for storage
// ============================================================================

/// A packed pair identifier combining relation ID and target entity.
///
/// This is used internally for efficient storage and comparison.
/// The high 32 bits are the relation's component ID, the low 32 bits
/// are the target entity's ID.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PairId(u64);

impl PairId {
    /// Create a pair ID from a relation component ID and target entity.
    #[must_use]
    pub const fn new(relation_id: u32, target_id: u32) -> Self {
        Self(((relation_id as u64) << 32) | (target_id as u64))
    }

    /// Get the relation component ID.
    #[must_use]
    pub const fn relation_id(self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Get the target entity ID.
    #[must_use]
    pub const fn target_id(self) -> u32 {
        self.0 as u32
    }

    /// Get the raw u64 representation.
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0
    }

    /// Create from raw u64 representation.
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// Create a wildcard pair that matches any target.
    #[must_use]
    pub const fn wildcard(relation_id: u32) -> Self {
        Self::new(relation_id, u32::MAX)
    }

    /// Check if this is a wildcard pair.
    #[must_use]
    pub const fn is_wildcard(self) -> bool {
        self.target_id() == u32::MAX
    }

    /// Check if this pair matches another (considering wildcards).
    #[must_use]
    pub fn matches(self, other: Self) -> bool {
        if self.relation_id() != other.relation_id() {
            return false;
        }
        self.is_wildcard() || other.is_wildcard() || self.target_id() == other.target_id()
    }
}

impl fmt::Debug for PairId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_wildcard() {
            write!(f, "PairId(rel:{}, target:*)", self.relation_id())
        } else {
            write!(
                f,
                "PairId(rel:{}, target:{})",
                self.relation_id(),
                self.target_id()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Generation;

    #[test]
    fn test_pair_creation() {
        let target = Entity::new(42, Generation::new());
        let pair: Pair<ChildOf> = Pair::new(target);

        assert_eq!(pair.target(), target);
    }

    #[test]
    fn test_pair_id_packing() {
        let pair_id = PairId::new(100, 42);

        assert_eq!(pair_id.relation_id(), 100);
        assert_eq!(pair_id.target_id(), 42);

        // Roundtrip through bits
        let bits = pair_id.to_bits();
        let recovered = PairId::from_bits(bits);
        assert_eq!(pair_id, recovered);
    }

    #[test]
    fn test_pair_id_wildcard() {
        let wildcard = PairId::wildcard(100);
        let specific = PairId::new(100, 42);
        let different_rel = PairId::new(200, 42);

        assert!(wildcard.is_wildcard());
        assert!(!specific.is_wildcard());

        // Wildcard matches any target with same relation
        assert!(wildcard.matches(specific));
        assert!(specific.matches(wildcard));

        // Different relations don't match
        assert!(!wildcard.matches(different_rel));
    }

    #[test]
    fn test_pair_equality() {
        let target = Entity::new(42, Generation::new());
        let pair1: Pair<ChildOf> = Pair::new(target);
        let pair2: Pair<ChildOf> = Pair::new(target);
        let pair3: Pair<ChildOf> = Pair::new(Entity::new(43, Generation::new()));

        assert_eq!(pair1, pair2);
        assert_ne!(pair1, pair3);
    }
}
