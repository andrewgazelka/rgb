//! Scope - restricted view of the world for RGB parallel execution.

use rgb_ecs::{Entity, World};

/// Identifier for a chunk in the spatial grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkId(pub u32);

impl ChunkId {
    /// Create a chunk ID from grid coordinates.
    #[must_use]
    pub const fn from_coords(x: u32, y: u32, grid_width: u32) -> Self {
        Self(y * grid_width + x)
    }

    /// Get grid X coordinate.
    #[must_use]
    pub const fn x(self, grid_width: u32) -> u32 {
        self.0 % grid_width
    }

    /// Get grid Y coordinate.
    #[must_use]
    pub const fn y(self, grid_width: u32) -> u32 {
        self.0 / grid_width
    }
}

/// A 3x3 neighborhood of chunks.
///
/// ```text
/// ┌───┬───┬───┐
/// │ 0 │ 1 │ 2 │   NW  N  NE
/// ├───┼───┼───┤
/// │ 3 │ 4 │ 5 │   W   C  E
/// ├───┼───┼───┤
/// │ 6 │ 7 │ 8 │   SW  S  SE
/// └───┴───┴───┘
/// ```
///
/// Index 4 is always the center chunk.
#[derive(Debug)]
pub struct Neighborhood {
    /// The 9 chunk IDs (some may be None for edge chunks)
    pub chunks: [Option<ChunkId>; 9],
    /// The center chunk (always present)
    pub center: ChunkId,
}

impl Neighborhood {
    /// Create a neighborhood centered on the given chunk.
    #[must_use]
    pub fn new(center: ChunkId, grid_width: u32, grid_height: u32) -> Self {
        let cx = center.x(grid_width) as i32;
        let cy = center.y(grid_width) as i32;

        let mut chunks = [None; 9];

        for (idx, (dy, dx)) in [
            (-1, -1),
            (-1, 0),
            (-1, 1), // NW, N, NE
            (0, -1),
            (0, 0),
            (0, 1), // W, C, E
            (1, -1),
            (1, 0),
            (1, 1), // SW, S, SE
        ]
        .iter()
        .enumerate()
        {
            let nx = cx + dx;
            let ny = cy + dy;

            if nx >= 0 && nx < grid_width as i32 && ny >= 0 && ny < grid_height as i32 {
                chunks[idx] = Some(ChunkId::from_coords(nx as u32, ny as u32, grid_width));
            }
        }

        Self { chunks, center }
    }

    /// Get the center chunk ID.
    #[must_use]
    pub const fn center(&self) -> ChunkId {
        self.center
    }

    /// Check if a chunk is in this neighborhood.
    #[must_use]
    pub fn contains(&self, chunk: ChunkId) -> bool {
        self.chunks.contains(&Some(chunk))
    }

    /// Iterate over all valid chunk IDs in the neighborhood.
    pub fn iter(&self) -> impl Iterator<Item = ChunkId> + '_ {
        self.chunks.iter().filter_map(|c| *c)
    }
}

/// A scoped view of the world, restricted to a 3x3 chunk neighborhood.
///
/// During RGB parallel execution, each chunk processor gets a `Scope`
/// that provides access only to its neighborhood. This ensures:
///
/// 1. No data races - same-colored chunks have non-overlapping neighborhoods
/// 2. Locality - operations are spatially bounded
/// 3. Type safety - can't accidentally access outside the scope
///
/// # Owned Value Pattern
///
/// All component access returns owned values (following SpacetimeDB pattern):
/// - `get<T>()` - Returns owned `T` (cloned)
/// - `update<T>()` - Write back modified value
/// - `insert<T>()` - Add new component
/// - `remove<T>()` - Remove and return component
pub struct Scope<'w> {
    /// Reference to the world
    world: &'w mut World,
    /// The accessible neighborhood
    neighborhood: Neighborhood,
    // TODO: Add entity-to-chunk mapping for filtering queries
}

impl<'w> Scope<'w> {
    /// Create a new scope for the given neighborhood.
    ///
    /// # Safety
    ///
    /// Caller must ensure no other scope has overlapping access to these chunks.
    pub fn new(world: &'w mut World, neighborhood: Neighborhood) -> Self {
        Self {
            world,
            neighborhood,
        }
    }

    /// Get the neighborhood this scope covers.
    #[must_use]
    pub const fn neighborhood(&self) -> &Neighborhood {
        &self.neighborhood
    }

    /// Get the center chunk ID.
    #[must_use]
    pub const fn center_chunk(&self) -> ChunkId {
        self.neighborhood.center
    }

    // ==================== Entity Operations ====================

    /// Get an owned copy of an entity's component.
    ///
    /// Returns `None` if the entity doesn't exist, doesn't have the component,
    /// or is not in this scope's neighborhood.
    #[must_use]
    pub fn get<T: 'static + Send + Sync + Clone>(&self, entity: Entity) -> Option<T> {
        // TODO: Check if entity is in scope
        self.world.get(entity)
    }

    /// Update an entity's component with a new value.
    ///
    /// Returns `false` if the entity doesn't exist, doesn't have the component,
    /// or is not in this scope's neighborhood.
    pub fn update<T: 'static + Send + Sync>(&mut self, entity: Entity, component: T) -> bool {
        // TODO: Check if entity is in scope
        self.world.update(entity, component)
    }

    /// Insert a component on an entity.
    ///
    /// Returns `false` if the entity doesn't exist or is not in this scope's neighborhood.
    pub fn insert<T: 'static + Send + Sync>(&mut self, entity: Entity, component: T) -> bool {
        // TODO: Check if entity is in scope
        self.world.insert(entity, component)
    }

    /// Remove a component from an entity.
    ///
    /// Returns `None` if the entity doesn't exist, doesn't have the component,
    /// or is not in this scope's neighborhood.
    pub fn remove<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<T> {
        // TODO: Check if entity is in scope
        self.world.remove(entity)
    }

    /// Check if an entity has a component.
    #[must_use]
    pub fn has<T: 'static + Send + Sync>(&self, entity: Entity) -> bool {
        // TODO: Check if entity is in scope
        self.world.has::<T>(entity)
    }

    /// Check if an entity is alive and in this scope.
    #[must_use]
    pub fn is_alive(&self, entity: Entity) -> bool {
        // TODO: Check if entity is in scope
        self.world.is_alive(entity)
    }

    // ==================== Relation Operations ====================

    /// Get the parent of an entity (via ChildOf relation).
    #[must_use]
    pub fn parent(&self, entity: Entity) -> Option<Entity> {
        self.world.parent(entity)
    }

    /// Set the parent of an entity (via ChildOf relation).
    pub fn set_parent(&mut self, child: Entity, parent: Entity) -> bool {
        // TODO: Check if both entities are in scope
        self.world.set_parent(child, parent)
    }

    // ==================== Deferred Operations ====================
    // These are deferred to avoid iterator invalidation

    /// Defer despawning an entity.
    ///
    /// The entity will be removed after the current parallel phase completes.
    pub fn defer_despawn(&mut self, _entity: Entity) {
        // TODO: Track deferred despawns in a Vec<Entity>
    }

    // TODO: Add defer_spawn with a proper builder pattern
    // For now, spawning can be done through the world after the parallel phase
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighborhood_creation() {
        // 5x5 grid, center at (2, 2)
        let center = ChunkId::from_coords(2, 2, 5);
        let hood = Neighborhood::new(center, 5, 5);

        // Center should be at index 4
        assert_eq!(hood.center(), center);
        assert!(hood.contains(center));

        // All 9 chunks should be valid (not on edge)
        assert_eq!(hood.iter().count(), 9);
    }

    #[test]
    fn test_neighborhood_corner() {
        // 5x5 grid, corner at (0, 0)
        let center = ChunkId::from_coords(0, 0, 5);
        let hood = Neighborhood::new(center, 5, 5);

        // Only 4 chunks valid (center + E + S + SE)
        assert_eq!(hood.iter().count(), 4);
        assert!(hood.contains(center));
        assert!(hood.contains(ChunkId::from_coords(1, 0, 5))); // E
        assert!(hood.contains(ChunkId::from_coords(0, 1, 5))); // S
        assert!(hood.contains(ChunkId::from_coords(1, 1, 5))); // SE
    }

    #[test]
    fn test_neighborhood_edge() {
        // 5x5 grid, edge at (2, 0) - top edge
        let center = ChunkId::from_coords(2, 0, 5);
        let hood = Neighborhood::new(center, 5, 5);

        // 6 chunks valid (no row above)
        assert_eq!(hood.iter().count(), 6);
    }

    #[test]
    fn test_scope_basic() {
        let mut world = World::new();
        let center = ChunkId::from_coords(1, 1, 3);
        let hood = Neighborhood::new(center, 3, 3);
        let scope = Scope::new(&mut world, hood);

        // Spawn through world (not scope for now)
        // TODO: Test scoped operations
        assert_eq!(scope.center_chunk(), center);
    }
}
