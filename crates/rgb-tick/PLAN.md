# rgb-tick Plan

## Purpose

Tick-based execution scheduler with RGB parallel phases and deferred mutation application.

## Execution Model

```
TICK N
═══════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────────┐
│ PHASE 0: Sequential Pre-Processing                                  │
│                                                                      │
│  1. Drain RPC queue from network layer                              │
│  2. Route RPCs to chunks based on spatial position                  │
│  3. Execute sequential RPCs (those with &mut Singleton)             │
│  4. Create immutable snapshot reference for parallel phase          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ PHASE 1: RED Chunks (Parallel)                                       │
│                                                                      │
│  rayon::scope(|s| {                                                 │
│      for chunk in world.chunks_by_color(Color::Red) {               │
│          s.spawn(|_| {                                              │
│              let region = chunk.neighborhood();  // 3x3 view        │
│              for rpc in chunk.drain_rpcs() {                        │
│                  rpc.execute(&region, &singletons);                 │
│              }                                                       │
│              chunk.tick_systems(&region, &singletons);              │
│          });                                                         │
│      }                                                               │
│  });  // implicit barrier                                            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ PHASE 2: GREEN Chunks (Parallel)                                     │
│  [Same structure as RED]                                             │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ PHASE 3: BLUE Chunks (Parallel)                                      │
│  [Same structure as RED]                                             │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ PHASE 4: Sequential Post-Processing                                  │
│                                                                      │
│  1. Collect deferred mutations from all chunks                      │
│  2. Sort for determinism (by entity ID, then operation type)        │
│  3. Apply mutations to world state:                                 │
│     a. Despawns                                                      │
│     b. Component removals                                            │
│     c. Component insertions                                          │
│     d. Spawns                                                        │
│  4. Handle entity chunk migrations                                   │
│  5. Execute post-tick sequential RPCs                               │
│  6. Commit tick to WAL                                              │
│  7. Generate observer deltas for subscriptions                      │
│  8. Increment tick counter                                          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Types

### `TickScheduler`

```rust
pub struct TickScheduler {
    /// Current tick number
    tick: u64,

    /// Thread pool for parallel execution
    thread_pool: rayon::ThreadPool,

    /// Pending RPCs not yet routed to chunks
    rpc_queue: crossbeam::channel::Receiver<IncomingRpc>,

    /// Sequential RPCs (those needing &mut Singleton)
    sequential_rpcs: Vec<Box<dyn SequentialRpc>>,
}

impl TickScheduler {
    pub fn run_tick(&mut self, world: &mut World) {
        // Phase 0: Pre-processing
        self.route_rpcs(world);
        self.execute_sequential_pre(world);

        // Phases 1-3: RGB parallel
        for color in [Color::Red, Color::Green, Color::Blue] {
            self.execute_parallel_phase(world, color);
        }

        // Phase 4: Post-processing
        self.apply_deferred(world);
        self.execute_sequential_post(world);
        self.commit_tick(world);

        self.tick += 1;
    }
}
```

### `DeferredMutations`

```rust
/// Thread-local collection of deferred mutations
pub struct DeferredMutations {
    /// Insertions: (entity, component_id, serialized_data)
    inserts: Vec<(Entity, ComponentId, Box<[u8]>)>,

    /// Removals: (entity, component_id)
    removals: Vec<(Entity, ComponentId)>,

    /// Spawns: list of components for new entity
    spawns: Vec<SpawnBundle>,

    /// Despawns: entities to remove
    despawns: Vec<Entity>,
}

impl DeferredMutations {
    /// Merge another set of mutations into this one
    pub fn merge(&mut self, other: DeferredMutations);

    /// Sort for deterministic application order
    pub fn sort(&mut self);

    /// Apply all mutations to world
    pub fn apply(self, world: &mut World);
}
```

### `Region` (Neighborhood View)

```rust
/// Read-only view of 3x3 chunk neighborhood
pub struct Region<'w> {
    /// Center chunk ID
    center: ChunkId,

    /// The 9 chunks (some may be None at world edges)
    chunks: [Option<&'w Chunk>; 9],

    /// Deferred mutations for this region
    deferred: RefCell<DeferredMutations>,

    /// Read-only singletons
    singletons: &'w SingletonStorage,
}

impl<'w> Region<'w> {
    // Query API
    pub fn query<Q: QueryData>(&self) -> RegionQuery<'w, Q>;
    pub fn find(&self, entity: Entity) -> Option<EntityRef<'w>>;
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T>;

    // Deferred mutation API
    pub fn defer_insert<T: Component>(&self, entity: Entity, component: T);
    pub fn defer_remove<T: Component>(&self, entity: Entity);
    pub fn defer_spawn(&self) -> DeferredEntityBuilder<'_>;
    pub fn defer_despawn(&self, entity: Entity);
}
```

## Chunk Migration

When an entity's position changes, it may need to move chunks:

```rust
impl TickScheduler {
    fn apply_deferred(&mut self, world: &mut World) {
        // Collect all mutations
        let mutations = world.drain_deferred();

        // Apply mutations
        for mutation in mutations {
            mutation.apply(world);
        }

        // Check for chunk migrations
        for entity in world.entities_with::<Position>() {
            let pos = world.get::<Position>(entity).unwrap();
            let current_chunk = world.chunk_of(entity);
            let correct_chunk = world.chunk_at(pos);

            if current_chunk != correct_chunk {
                world.migrate_entity(entity, current_chunk, correct_chunk);
            }
        }
    }
}
```

## Implementation Steps

1. Define `DeferredMutations` struct
2. Implement mutation collection and merging
3. Implement deterministic sorting
4. Implement mutation application
5. Define `Region` struct with deferred API
6. Implement `TickScheduler` skeleton
7. Implement RPC routing to chunks
8. Implement parallel phase execution with rayon
9. Implement chunk migration detection
10. Add tick counter and timing

## Files

```
src/
├── lib.rs              # Re-exports
├── scheduler.rs        # TickScheduler
├── deferred.rs         # DeferredMutations
├── region.rs           # Region (neighborhood view)
├── phase.rs            # RGB phase execution
└── migration.rs        # Entity chunk migration
```

## Dependencies

- `rgb-ecs` (World, components)
- `rgb-spatial` (chunks, colors)
- `rgb-query` (RegionQuery)
- `rayon` (parallel execution)
- `crossbeam` (channels for RPC queue)

## Thread Safety

- `Region` uses `RefCell<DeferredMutations>` - single-threaded access within parallel task
- Each chunk processes independently
- Mutations merged after parallel phase (sequential)
- No locks needed during parallel execution

## Determinism

For replay and debugging:

1. RPCs sorted by (chunk_id, rpc_sequence_number)
2. Deferred mutations sorted by (entity_id, operation_type)
3. Tick-seeded RNG for any randomness
4. Consistent floating-point operations
