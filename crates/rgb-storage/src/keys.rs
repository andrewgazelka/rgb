//! Key encoding for versioned storage.
//!
//! Keys are designed for efficient prefix scanning:
//! - Scan all components of an entity: prefix = entity_bits
//! - Scan specific component across entities: requires secondary index
//!
//! # Key Format
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │  ComponentKey (12 bytes)                                   │
//! ├────────────────────────────────────────────────────────────┤
//! │  entity_bits: u64     (8 bytes) - Entity packed bits       │
//! │  component_id: u32    (4 bytes) - Component type ID        │
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! Entity bits are packed as: (generation << 32) | id
//! This ensures the same entity ID with different generations are different keys.

use bytemuck::{Pod, Zeroable};
use rgb_ecs::{ComponentId, Entity};

/// Compile-time endianness check.
/// We only support little-endian for direct memory serialization.
const _: () = {
    #[cfg(not(target_endian = "little"))]
    compile_error!("rgb-storage only supports little-endian architectures");
};

/// A key identifying a specific component on a specific entity.
///
/// This is stored directly in Nebari as the key bytes.
/// Uses `#[repr(C, packed)]` to avoid padding between fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
#[repr(C, packed)]
pub struct ComponentKey {
    /// Entity packed as bits (includes generation).
    entity_bits: u64,
    /// Component type ID.
    component_id: u32,
}

impl ComponentKey {
    /// Create a new component key.
    #[inline]
    #[must_use]
    pub const fn new(entity: Entity, component_id: ComponentId) -> Self {
        Self {
            entity_bits: entity.to_bits(),
            component_id: component_id.as_raw(),
        }
    }

    /// Get the entity from this key.
    #[inline]
    #[must_use]
    pub const fn entity(&self) -> Entity {
        Entity::from_bits(self.entity_bits)
    }

    /// Get the component ID from this key.
    #[inline]
    #[must_use]
    pub const fn component_id(&self) -> ComponentId {
        ComponentId::from_raw(self.component_id)
    }

    /// Convert to bytes for storage.
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    /// Convert from bytes.
    #[inline]
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != std::mem::size_of::<Self>() {
            return None;
        }
        Some(*bytemuck::from_bytes(bytes))
    }

    /// Create a prefix key for scanning all components of an entity.
    #[inline]
    #[must_use]
    pub fn entity_prefix(entity: Entity) -> [u8; 8] {
        entity.to_bits().to_le_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rgb_ecs::{Entity, Generation};

    #[test]
    fn test_key_roundtrip() {
        let entity = Entity::new(42, Generation::new());
        let component_id = ComponentId::from_raw(123);
        let key = ComponentKey::new(entity, component_id);

        let bytes = key.as_bytes();
        assert_eq!(bytes.len(), 12);

        let recovered = ComponentKey::from_bytes(bytes).unwrap();
        assert_eq!(key, recovered);
        assert_eq!(recovered.entity(), entity);
        assert_eq!(recovered.component_id(), component_id);
    }

    #[test]
    fn test_key_ordering() {
        // Keys should sort by entity first, then component
        let e1 = Entity::new(1, Generation::new());
        let e2 = Entity::new(2, Generation::new());
        let c1 = ComponentId::from_raw(1);
        let c2 = ComponentId::from_raw(2);

        let k1 = ComponentKey::new(e1, c1);
        let k2 = ComponentKey::new(e1, c2);
        let k3 = ComponentKey::new(e2, c1);

        // Same entity, different components
        assert!(k1 < k2);
        // Different entities
        assert!(k2 < k3);
    }
}
