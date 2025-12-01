# rgb-query Plan

## Purpose

Provide Flecs-style query API for accessing entity components within a region (3x3 chunk neighborhood).

## Key Constraint

**All entity component access is `&T` (read-only).**

There is NO `&mut T` for entity components. Mutations are deferred.

## Core Types

### `Query<Q>` - Component Query Descriptor

```rust
/// Describes what components to fetch
pub trait QueryData {
    type Item<'w>;

    /// Component IDs this query reads
    fn component_ids(registry: &ComponentRegistry) -> Vec<ComponentId>;

    /// Fetch from archetype row
    unsafe fn fetch<'w>(archetype: &'w Archetype, row: usize) -> Self::Item<'w>;
}

// Implementation for single component reference
impl<T: Component> QueryData for &T {
    type Item<'w> = &'w T;
    // ...
}

// Implementation for tuples
impl<A: QueryData, B: QueryData> QueryData for (A, B) {
    type Item<'w> = (A::Item<'w>, B::Item<'w>);
    // ...
}
```

### `QueryFilter` - Optional Filters

```rust
/// Filter archetypes
pub trait QueryFilter {
    fn matches(archetype: &Archetype, registry: &ComponentRegistry) -> bool;
}

/// Entity must have component T
pub struct With<T>(PhantomData<T>);

/// Entity must NOT have component T
pub struct Without<T>(PhantomData<T>);

impl<T: Component> QueryFilter for With<T> {
    fn matches(archetype: &Archetype, registry: &ComponentRegistry) -> bool {
        let id = registry.get_id::<T>().unwrap();
        archetype.contains(id)
    }
}
```

### `RegionQuery` - Query Over Neighborhood

```rust
/// Query iterator over a 3x3 chunk region
pub struct RegionQuery<'w, Q: QueryData, F: QueryFilter = ()> {
    chunks: &'w [Option<&'w Chunk>; 9],
    current_chunk: usize,
    current_archetype: usize,
    current_row: usize,
    _marker: PhantomData<(Q, F)>,
}

impl<'w, Q: QueryData, F: QueryFilter> Iterator for RegionQuery<'w, Q, F> {
    type Item = (Entity, Q::Item<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate through chunks -> archetypes -> rows
    }
}
```

## API Usage

```rust
// In an RPC handler
fn on_explosion(ctx: &RpcContext, origin: Vec3, radius: f32) {
    let region = ctx.region();

    // Query all entities with Position and Health
    for (entity, (pos, health)) in region.query::<(&Position, &Health)>() {
        if pos.distance(origin) < radius {
            let damage = calculate_damage(pos, origin, radius);
            region.defer_insert(entity, Damage(damage));
        }
    }

    // Query with filter
    for (entity, pos) in region.query::<&Position>().filter::<With<Player>>() {
        // Only players
    }
}
```

## Implementation Steps

1. Define `QueryData` trait
2. Implement for `&T`
3. Implement for tuples up to 8 elements
4. Define `QueryFilter` trait
5. Implement `With<T>`, `Without<T>`
6. Implement `ArchetypeQuery` (single archetype iterator)
7. Implement `RegionQuery` (multi-chunk iterator)
8. Add query builder API

## Files

```
src/
├── lib.rs          # Re-exports
├── data.rs         # QueryData trait and impls
├── filter.rs       # QueryFilter trait and impls
├── iter.rs         # Iterator implementations
└── region.rs       # RegionQuery
```

## Dependencies

- `rgb-ecs` (archetypes, components)
- `rgb-spatial` (chunks, regions)

## Non-Goals

- `&mut T` access for entities (use deferred mutations)
- Cross-region queries (only 3x3 neighborhood)
- Parallel iteration within query (handled at chunk level)
