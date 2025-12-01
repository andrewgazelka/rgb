//! Query system for iterating over entities with specific component patterns.
//!
//! This module provides a **runtime query builder** inspired by Flecs-Rust.
//! Queries are built using method chaining, not type-level generics.
//!
//! # Design Goals
//!
//! - **FFI-friendly**: Can be exposed to Lua, WASM, other languages
//! - **Simple errors**: No 500-line generic error messages
//! - **Dynamic composition**: Build queries from runtime data
//! - **Owned values**: Components are cloned (RGB's SpacetimeDB-style access)
//!
//! # Basic Usage
//!
//! ```ignore
//! let query = world.query()
//!     .with::<Position>()
//!     .with::<Velocity>()
//!     .build();
//!
//! for row in query.iter(&world) {
//!     let entity = row.entity();
//!     let pos: Position = row.get::<Position>();
//!     let vel: Velocity = row.get::<Velocity>();
//!     println!("{entity:?} at ({}, {})", pos.x, pos.y);
//! }
//! ```
//!
//! # Query Combinators
//!
//! - `.with::<T>()` - Entity must have component T (fetches data)
//! - `.optional::<T>()` - Fetch T if present (returns Option)
//! - `.without::<T>()` - Entity must NOT have component T
//! - `.filter::<T>()` - Entity must have T, but don't fetch data
//!
//! # Example with Filters
//!
//! ```ignore
//! let query = world.query()
//!     .with::<Position>()
//!     .with::<Velocity>()
//!     .filter::<Enemy>()      // Must be Enemy, but don't fetch Enemy data
//!     .without::<Dead>()       // Exclude dead entities
//!     .build();
//! ```

use crate::{
    World,
    archetype::{Archetype, ArchetypeId},
    component::ComponentId,
    entity::Entity,
};

// ============================================================================
// Term Types
// ============================================================================

/// How a component is accessed in a query term.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermAccess {
    /// Fetch the component data (cloned).
    Fetch,
    /// Component is optional - fetch if present.
    Optional,
    /// Filter only - entity must have component, but don't fetch.
    Filter,
    /// Negation - entity must NOT have component.
    Without,
}

/// A single term in a query.
#[derive(Clone, Debug)]
pub struct QueryTerm {
    /// Component ID for this term.
    pub component_id: ComponentId,
    /// How the component is accessed.
    pub access: TermAccess,
}

// ============================================================================
// QueryBuilder - Runtime Builder Pattern
// ============================================================================

/// Builder for constructing queries at runtime.
///
/// This is the preferred way to build queries - simple method chaining
/// without complex type-level generics.
pub struct QueryBuilder<'w> {
    world: &'w World,
    terms: Vec<QueryTerm>,
}

impl<'w> QueryBuilder<'w> {
    /// Create a new query builder.
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            terms: Vec::new(),
        }
    }

    /// Add a required component that will be fetched.
    ///
    /// Entity must have this component, and its value will be available
    /// via `QueryRow::get::<T>()`.
    #[must_use]
    pub fn with<T: 'static + Send + Sync + Clone>(mut self) -> Self {
        if let Some(comp_id) = self.world.component_id::<T>() {
            self.terms.push(QueryTerm {
                component_id: comp_id,
                access: TermAccess::Fetch,
            });
        }
        self
    }

    /// Add an optional component.
    ///
    /// If entity has this component, its value will be available via
    /// `QueryRow::get_optional::<T>()`. Query still matches if component is missing.
    #[must_use]
    pub fn optional<T: 'static + Send + Sync + Clone>(mut self) -> Self {
        if let Some(comp_id) = self.world.component_id::<T>() {
            self.terms.push(QueryTerm {
                component_id: comp_id,
                access: TermAccess::Optional,
            });
        }
        self
    }

    /// Add a filter - entity must have component, but don't fetch data.
    ///
    /// Useful for tag/marker components or when you only need to check existence.
    #[must_use]
    pub fn filter<T: 'static + Send + Sync>(mut self) -> Self {
        if let Some(comp_id) = self.world.component_id::<T>() {
            self.terms.push(QueryTerm {
                component_id: comp_id,
                access: TermAccess::Filter,
            });
        }
        self
    }

    /// Exclude entities that have this component.
    #[must_use]
    pub fn without<T: 'static + Send + Sync>(mut self) -> Self {
        if let Some(comp_id) = self.world.component_id::<T>() {
            self.terms.push(QueryTerm {
                component_id: comp_id,
                access: TermAccess::Without,
            });
        }
        self
    }

    /// Build the query.
    ///
    /// Pre-computes matching archetypes for efficient iteration.
    #[must_use]
    pub fn build(self) -> Query {
        // Pre-compute matching archetypes
        let matching_archetypes: Vec<ArchetypeId> = self
            .world
            .archetypes()
            .iter()
            .filter(|arch| {
                for term in &self.terms {
                    let has_component = arch.contains(term.component_id);

                    match term.access {
                        TermAccess::Without => {
                            if has_component {
                                return false;
                            }
                        }
                        TermAccess::Optional => {
                            // Optional always matches
                        }
                        TermAccess::Fetch | TermAccess::Filter => {
                            if !has_component {
                                return false;
                            }
                        }
                    }
                }
                true
            })
            .map(|arch| arch.id())
            .collect();

        Query {
            terms: self.terms,
            matching_archetypes,
        }
    }
}

// ============================================================================
// Query - Executable Query
// ============================================================================

/// An executable query over entities.
///
/// Queries cache matching archetypes for efficient iteration.
pub struct Query {
    terms: Vec<QueryTerm>,
    matching_archetypes: Vec<ArchetypeId>,
}

impl Query {
    /// Get the number of matching archetypes.
    #[must_use]
    pub fn archetype_count(&self) -> usize {
        self.matching_archetypes.len()
    }

    /// Get the query terms.
    #[must_use]
    pub fn terms(&self) -> &[QueryTerm] {
        &self.terms
    }

    /// Iterate over all matching entities.
    pub fn iter<'w, 'q>(&'q self, world: &'w World) -> QueryIter<'w, 'q> {
        QueryIter::new(world, self)
    }

    /// Execute a closure for each matching entity.
    pub fn each<F>(&self, world: &World, mut f: F)
    where
        F: FnMut(QueryRow<'_>),
    {
        for row in self.iter(world) {
            f(row);
        }
    }
}

impl core::fmt::Debug for Query {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Query")
            .field("term_count", &self.terms.len())
            .field("matching_archetypes", &self.matching_archetypes.len())
            .finish()
    }
}

// ============================================================================
// QueryIter - Iterator Over Query Results
// ============================================================================

/// Iterator over query results.
pub struct QueryIter<'w, 'q> {
    world: &'w World,
    query: &'q Query,
    archetype_idx: usize,
    row: usize,
}

impl<'w, 'q> QueryIter<'w, 'q> {
    fn new(world: &'w World, query: &'q Query) -> Self {
        Self {
            world,
            query,
            archetype_idx: 0,
            row: 0,
        }
    }
}

impl<'w, 'q> Iterator for QueryIter<'w, 'q> {
    type Item = QueryRow<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.archetype_idx >= self.query.matching_archetypes.len() {
                return None;
            }

            let arch_id = self.query.matching_archetypes[self.archetype_idx];
            let archetype = self.world.archetypes().get(arch_id)?;

            if self.row >= archetype.len() {
                self.archetype_idx += 1;
                self.row = 0;
                continue;
            }

            let entity = archetype.entities()[self.row];
            let row = self.row;
            self.row += 1;

            return Some(QueryRow {
                world: self.world,
                archetype,
                entity,
                row,
            });
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut remaining = 0;

        for i in self.archetype_idx..self.query.matching_archetypes.len() {
            let arch_id = self.query.matching_archetypes[i];
            if let Some(archetype) = self.world.archetypes().get(arch_id) {
                if i == self.archetype_idx {
                    remaining += archetype.len().saturating_sub(self.row);
                } else {
                    remaining += archetype.len();
                }
            }
        }

        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for QueryIter<'_, '_> {}

// ============================================================================
// QueryRow - Single Row Access
// ============================================================================

/// A single row from a query result.
///
/// Provides access to the entity and its components.
pub struct QueryRow<'w> {
    world: &'w World,
    archetype: &'w Archetype,
    entity: Entity,
    row: usize,
}

impl<'w> QueryRow<'w> {
    /// Get the entity for this row.
    #[must_use]
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Get a required component value (cloned).
    ///
    /// # Panics
    ///
    /// Panics if the component is not present. Use `get_optional` for
    /// components that may not exist.
    #[must_use]
    pub fn get<T: 'static + Send + Sync + Clone>(&self) -> T {
        self.get_optional::<T>()
            .expect("Component not present - use get_optional() for optional components")
    }

    /// Get an optional component value (cloned).
    ///
    /// Returns `None` if the entity doesn't have this component.
    #[must_use]
    pub fn get_optional<T: 'static + Send + Sync + Clone>(&self) -> Option<T> {
        let comp_id = self.world.component_id::<T>()?;

        if !self.archetype.contains(comp_id) {
            return None;
        }

        // SAFETY: We verified the archetype contains the component
        unsafe {
            self.archetype
                .get_component::<T>(comp_id, self.row)
                .cloned()
        }
    }

    /// Check if entity has a component.
    #[must_use]
    pub fn has<T: 'static + Send + Sync>(&self) -> bool {
        self.world
            .component_id::<T>()
            .is_some_and(|comp_id| self.archetype.contains(comp_id))
    }

    /// Get the world reference.
    #[must_use]
    pub fn world(&self) -> &'w World {
        self.world
    }
}

impl core::fmt::Debug for QueryRow<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QueryRow")
            .field("entity", &self.entity)
            .finish()
    }
}

// ============================================================================
// Tests
// ============================================================================

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

    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    struct Enemy;

    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    struct Dead;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Health(u32);

    #[test]
    fn test_simple_query() {
        let mut world = World::new();

        let e1 = world.spawn(Position { x: 1.0, y: 2.0 });
        world.insert(e1, Velocity { x: 0.1, y: 0.2 });

        let e2 = world.spawn(Position { x: 3.0, y: 4.0 });
        world.insert(e2, Velocity { x: 0.3, y: 0.4 });

        // Entity with only Position (no Velocity)
        let _e3 = world.spawn(Position { x: 5.0, y: 6.0 });

        let query = world.query().with::<Position>().with::<Velocity>().build();

        assert_eq!(query.iter(&world).count(), 2);
    }

    #[test]
    fn test_query_get_components() {
        let mut world = World::new();

        let e1 = world.spawn(Position { x: 1.0, y: 2.0 });
        world.insert(e1, Velocity { x: 0.1, y: 0.2 });

        let query = world.query().with::<Position>().with::<Velocity>().build();

        for row in query.iter(&world) {
            let pos: Position = row.get();
            let vel: Velocity = row.get();

            assert_eq!(pos.x, 1.0);
            assert_eq!(vel.x, 0.1);
        }
    }

    #[test]
    fn test_optional_query() {
        let mut world = World::new();

        let e1 = world.spawn(Position { x: 1.0, y: 2.0 });
        world.insert(e1, Velocity { x: 0.1, y: 0.2 });

        let _e2 = world.spawn(Position { x: 3.0, y: 4.0 });
        // e2 has no Velocity

        let query = world
            .query()
            .with::<Position>()
            .optional::<Velocity>()
            .build();

        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 2);

        let with_vel = results
            .iter()
            .filter(|r| r.get_optional::<Velocity>().is_some())
            .count();
        let without_vel = results
            .iter()
            .filter(|r| r.get_optional::<Velocity>().is_none())
            .count();

        assert_eq!(with_vel, 1);
        assert_eq!(without_vel, 1);
    }

    #[test]
    fn test_filter() {
        let mut world = World::new();

        let e1 = world.spawn(Position { x: 1.0, y: 2.0 });
        world.insert(e1, Enemy);

        let _e2 = world.spawn(Position { x: 3.0, y: 4.0 });
        // e2 is not an Enemy

        let query = world.query().with::<Position>().filter::<Enemy>().build();

        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity(), e1);
    }

    #[test]
    fn test_without() {
        let mut world = World::new();

        let e1 = world.spawn(Position { x: 1.0, y: 2.0 });
        world.insert(e1, Dead);

        let e2 = world.spawn(Position { x: 3.0, y: 4.0 });
        // e2 is not Dead

        let query = world.query().with::<Position>().without::<Dead>().build();

        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity(), e2);
    }

    #[test]
    fn test_combined_filters() {
        let mut world = World::new();

        // e1: Position + Enemy (should match)
        let e1 = world.spawn(Position { x: 1.0, y: 2.0 });
        world.insert(e1, Enemy);

        // e2: Position + Enemy + Dead (should NOT match)
        let e2 = world.spawn(Position { x: 3.0, y: 4.0 });
        world.insert(e2, Enemy);
        world.insert(e2, Dead);

        // e3: Position only (should NOT match - no Enemy)
        let _e3 = world.spawn(Position { x: 5.0, y: 6.0 });

        let query = world
            .query()
            .with::<Position>()
            .filter::<Enemy>()
            .without::<Dead>()
            .build();

        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity(), e1);
    }

    #[test]
    fn test_query_each() {
        let mut world = World::new();

        world.spawn(Position { x: 1.0, y: 2.0 });
        world.spawn(Position { x: 3.0, y: 4.0 });

        let query = world.query().with::<Position>().build();

        let mut count = 0;
        query.each(&world, |_row| {
            count += 1;
        });

        assert_eq!(count, 2);
    }

    #[test]
    fn test_query_row_has() {
        let mut world = World::new();

        let e1 = world.spawn(Position { x: 1.0, y: 2.0 });
        world.insert(e1, Enemy);

        let query = world.query().with::<Position>().build();

        for row in query.iter(&world) {
            assert!(row.has::<Position>());
            assert!(row.has::<Enemy>());
            assert!(!row.has::<Dead>());
        }
    }
}
