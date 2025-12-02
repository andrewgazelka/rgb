//! ScopedWorld - Safe boundary-checking wrapper for Flecs stages

use flecs_ecs::prelude::*;

use crate::region::{Position, chebyshev_distance};

/// Error returned when accessing entities outside the allowed scope
#[derive(Debug, Clone, thiserror::Error)]
pub enum ScopeError {
    /// Entity has no Position component
    #[error("entity has no Position component")]
    NoPosition,

    /// Entity is outside the allowed chunk neighborhood
    #[error(
        "entity at chunk ({entity_chunk_x}, {entity_chunk_z}) is outside bounds of center ({center_chunk_x}, {center_chunk_z})"
    )]
    OutOfBounds {
        entity_chunk_x: i32,
        entity_chunk_z: i32,
        center_chunk_x: i32,
        center_chunk_z: i32,
    },

    /// Component not found on entity
    #[error("component not found on entity")]
    ComponentNotFound,
}

/// A scoped view into the world that validates chunk boundaries.
///
/// During parallel execution, each chunk processor gets a `ScopedWorld`
/// that can only access entities within ±1 chunk (Chebyshev distance) of the center.
///
/// This is safe because 3-coloring ensures no two adjacent chunks run in parallel
/// within the same color phase.
pub struct ScopedWorld<'w> {
    /// The Flecs stage for this thread
    stage: WorldRef<'w>,
    /// Center chunk coordinates
    center_chunk: (i32, i32),
    /// Maximum Chebyshev distance allowed (default: 1)
    max_distance: i32,
}

impl<'w> ScopedWorld<'w> {
    /// Create a new ScopedWorld from a Flecs stage
    ///
    /// # Arguments
    /// * `stage` - The Flecs stage for this thread (from `world.stage(thread_id)`)
    /// * `center_chunk` - The (x, z) coordinates of the center chunk
    #[must_use]
    pub fn new(stage: WorldRef<'w>, center_chunk: (i32, i32)) -> Self {
        Self {
            stage,
            center_chunk,
            max_distance: 1,
        }
    }

    /// Create a ScopedWorld with custom max distance
    #[must_use]
    pub fn with_max_distance(
        stage: WorldRef<'w>,
        center_chunk: (i32, i32),
        max_distance: i32,
    ) -> Self {
        Self {
            stage,
            center_chunk,
            max_distance,
        }
    }

    /// Get the center chunk coordinates
    #[must_use]
    pub fn center_chunk(&self) -> (i32, i32) {
        self.center_chunk
    }

    /// Get the underlying stage
    #[must_use]
    pub fn stage(&self) -> &WorldRef<'w> {
        &self.stage
    }

    /// Validate that an entity is within the allowed chunk neighborhood
    fn validate_in_bounds(&self, entity: EntityView<'_>) -> Result<(), ScopeError> {
        // Get entity's position
        let Some(pos) = entity.try_get::<&Position>(|p| *p) else {
            return Err(ScopeError::NoPosition);
        };

        let entity_chunk = pos.chunk_coords();
        let dist = chebyshev_distance(self.center_chunk, entity_chunk);

        if dist > self.max_distance {
            return Err(ScopeError::OutOfBounds {
                entity_chunk_x: entity_chunk.0,
                entity_chunk_z: entity_chunk.1,
                center_chunk_x: self.center_chunk.0,
                center_chunk_z: self.center_chunk.1,
            });
        }

        Ok(())
    }

    /// Get a component from an entity (validates bounds)
    ///
    /// Returns an owned clone of the component value.
    pub fn get<T>(&self, entity: EntityView<'_>) -> Result<T, ScopeError>
    where
        T: ComponentId + DataComponent + Clone,
        T::UnderlyingType: Clone,
    {
        self.validate_in_bounds(entity)?;

        entity
            .try_get::<&T>(|c| c.clone())
            .ok_or(ScopeError::ComponentNotFound)
    }

    /// Set a component on an entity (validates bounds, deferred)
    ///
    /// The set operation is deferred until `readonly_end()` is called.
    pub fn set<T>(&self, entity: EntityView<'_>, value: T) -> Result<(), ScopeError>
    where
        T: ComponentId + ComponentType<Struct>,
    {
        self.validate_in_bounds(entity)?;

        // Use the stage's deferred operations
        entity.set(value);
        Ok(())
    }

    /// Spawn a new entity (always allowed, uses deferred operations)
    ///
    /// The spawned entity will be created when `readonly_end()` is called.
    pub fn spawn(&self) -> EntityView<'_> {
        self.stage.entity()
    }

    /// Check if an entity is within bounds without accessing components
    pub fn is_in_bounds(&self, entity: EntityView<'_>) -> Result<bool, ScopeError> {
        match self.validate_in_bounds(entity) {
            Ok(()) => Ok(true),
            Err(ScopeError::OutOfBounds { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoped_world_in_bounds() {
        let world = World::new();

        // Create an entity at chunk (0, 0)
        let entity = world.entity().set(Position::new(8.0, 64.0, 8.0)); // Center of chunk (0, 0)

        // ScopedWorld centered at (0, 0) should allow access
        let scoped = ScopedWorld::new((&world).world(), (0, 0));
        assert!(scoped.get::<Position>(entity).is_ok());
    }

    #[test]
    fn test_scoped_world_neighbor_access() {
        let world = World::new();

        // Entity at chunk (1, 0) - neighbor of (0, 0)
        let entity = world.entity().set(Position::new(24.0, 64.0, 8.0)); // chunk (1, 0)

        // Should be accessible from center (0, 0)
        let scoped = ScopedWorld::new((&world).world(), (0, 0));
        assert!(scoped.get::<Position>(entity).is_ok());

        // Should also be accessible from diagonal neighbor
        let scoped_diag = ScopedWorld::new((&world).world(), (0, 1));
        assert!(scoped_diag.get::<Position>(entity).is_ok());
    }

    #[test]
    fn test_scoped_world_out_of_bounds() {
        let world = World::new();

        // Entity at chunk (5, 5) - far from (0, 0)
        let entity = world.entity().set(Position::new(88.0, 64.0, 88.0)); // chunk (5, 5)

        // Should NOT be accessible from center (0, 0)
        let scoped = ScopedWorld::new((&world).world(), (0, 0));
        let result = scoped.get::<Position>(entity);
        assert!(matches!(result, Err(ScopeError::OutOfBounds { .. })));
    }

    #[test]
    fn test_scoped_world_no_position() {
        let world = World::new();

        // Entity without Position
        let entity = world.entity();

        let scoped = ScopedWorld::new((&world).world(), (0, 0));
        let result = scoped.get::<Position>(entity);
        assert!(matches!(result, Err(ScopeError::NoPosition)));
    }
}
