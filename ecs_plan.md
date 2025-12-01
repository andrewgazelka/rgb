# RGB ECS: Event-Driven Architecture with Spatial Parallelism

## Goal

Combine evenio-style event-driven architecture with RGB spatial parallelism for a Minecraft server that is both **composable** (event handlers) and **parallel** (spatial partitioning).

## Current State

- Manual system calls in `systems::tick()`
- RGB coloring exists in `rgb-spatial` but isn't used for parallelism
- Packet handling is imperative: decode packet → mutate state directly

## Proposed Architecture

### Core Concept: Spatial Events

Events are tagged with a **cell color** based on the entity they target. Events for same-colored cells can run in parallel since they never share edges.

```
Tick N:
┌─────────────────────────────────────────────────────────────────┐
│  Phase 0: Global Events (sequential)                            │
│           - NetworkIngress, Tick, etc.                          │
├─────────────────────────────────────────────────────────────────┤
│  Phase 1: RED cell events (parallel within color)               │
│  Phase 2: Barrier                                               │
│  Phase 3: GREEN cell events (parallel within color)             │
│  Phase 4: Barrier                                               │
│  Phase 5: BLUE cell events (parallel within color)              │
│  Phase 6: Barrier                                               │
├─────────────────────────────────────────────────────────────────┤
│  Phase 7: Global Events (sequential)                            │
│           - NetworkEgress, Commit, etc.                         │
└─────────────────────────────────────────────────────────────────┘
```

### Event Types

```rust
/// Global event - processed sequentially, sees entire world
#[derive(GlobalEvent)]
struct TickEvent { delta_time: f32 }

/// Spatial event - processed in RGB phases, only sees local cell + neighbors
#[derive(SpatialEvent)]
struct MoveEvent {
    dx: f64,
    dy: f64,
    dz: f64
}

/// Targeted event - sent to specific entity, scheduled by entity's cell color
#[derive(TargetedEvent)]
struct DamageEvent { amount: f32 }
```

### Handler Registration

```rust
// Global handler - runs once per tick
world.add_handler(|_: Receiver<TickEvent>, mut time: Single<&mut WorldTime>| {
    time.tick();
});

// Spatial handler - runs for entities in current color phase
// Can only access entities in same cell or edge-adjacent cells
world.add_handler(|event: Receiver<MoveEvent>,
                   mut pos: Fetcher<&mut Position>,
                   neighbors: Neighbors<&Position>| {
    let entity = event.target();
    let mut p = pos.get_mut(entity)?;

    // Update position locally
    p.x += event.dx;
    p.y += event.dy;
    p.z += event.dz;

    // Can read neighbor positions (for collision, etc.)
    for (neighbor_entity, neighbor_pos) in neighbors.iter() {
        // ...
    }
});

// Component-filtered handler
world.add_handler(|event: Receiver<MoveEvent>,
                   _: With<Collider>,
                   pos: Fetcher<&Position>,
                   physics: Fetcher<&mut PhysicsBody>| {
    // Only runs for entities with Collider component
});
```

### RPC Macro (Packet → Event)

```rust
#[rpc(MovePlayerPos = 0x1D)]
fn handle_move(x: f64, y: f64, z: f64) -> MoveEvent {
    MoveEvent { dx: x, dy: y, dz: z }
}

// Generates:
// 1. MoveEvent struct with x, y, z fields
// 2. Packet decoder that parses 0x1D and emits MoveEvent to entity
// 3. Registration in packet dispatch table
```

### Cell Color from Position (Computed, Not Stored)

Cell color is **deterministic** from position - just arithmetic, no storage needed:

```rust
const CELL_SHIFT: u32 = 4; // 16-block cells (same as chunks)

/// Compute cell color from world position - O(1) bit math
#[inline]
fn cell_color(x: f64, z: f64) -> Color {
    let cx = (x as i32) >> CELL_SHIFT;
    let cz = (z as i32) >> CELL_SHIFT;
    // (cx + cz) mod 3, handling negatives
    match ((cx + cz) % 3 + 3) % 3 {
        0 => Color::Red,
        1 => Color::Green,
        _ => Color::Blue,
    }
}

// Position is just coordinates - color computed on demand
#[derive(Component)]
struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position {
    #[inline]
    fn color(&self) -> Color {
        cell_color(self.x, self.z)
    }
}
```

A cast + bit shift + mod is trivial - no reason to cache.

### Event Queue Structure

```rust
struct EventQueue {
    /// Global events - processed first and last
    global: VecDeque<GlobalEventItem>,

    /// Spatial events bucketed by color
    red: VecDeque<SpatialEventItem>,
    green: VecDeque<SpatialEventItem>,
    blue: VecDeque<SpatialEventItem>,
}

impl EventQueue {
    fn push_spatial<E: SpatialEvent>(&mut self, target: Entity, pos: &Position, event: E) {
        let color = pos.color(); // Computed, not stored
        let item = SpatialEventItem { target, event: Box::new(event) };

        match color {
            Color::Red => self.red.push_back(item),
            Color::Green => self.green.push_back(item),
            Color::Blue => self.blue.push_back(item),
        }
    }
}
```

### Parallel Execution

```rust
fn flush_tick(world: &mut World) {
    // Phase 0: Global pre-tick events
    world.flush_global_events();

    // Phases 1-6: RGB spatial events
    for color in Color::ALL {
        // All RED (or GREEN or BLUE) cells can run in parallel
        // because no two same-colored cells share an edge
        rayon::scope(|s| {
            for cell in world.grid.cells_by_color(color) {
                s.spawn(|_| {
                    // Process all events for entities in this cell
                    cell.flush_events(world);
                });
            }
        });
        // Implicit barrier between colors
    }

    // Phase 7: Global post-tick events
    world.flush_global_events();
}
```

## Tradeoffs

### Advantages

1. **Composability**: Handlers are decoupled, easy to add/remove behavior
2. **Parallelism**: Same-colored cells run in parallel with zero synchronization
3. **Type Safety**: Events are strongly typed, handlers have clear signatures
4. **Locality**: Spatial handlers only see local data, cache-friendly
5. **Extensibility**: New packets just need `#[rpc]` annotation

### Disadvantages

1. **Complexity**: More concepts than simple systems (events, handlers, spatial scopes)
2. **Cross-Cell Interactions**: Actions spanning multiple cells need special handling
3. **Event Ordering**: Within a color phase, event order is non-deterministic
4. **Memory Overhead**: Event queues per color, handler metadata

### Open Questions

1. **Cross-cell movement**: When entity moves from RED cell to GREEN cell mid-tick, which phase processes it?
   - Option A: Process in source cell's phase, update cell assignment at barrier
   - Option B: Defer cross-cell moves to next tick
   - Option C: Use double-buffering for position updates

2. **Global state access**: How do spatial handlers read global state (WorldTime, etc.)?
   - Option A: Pass as read-only reference to all handlers
   - Option B: Snapshot global state at tick start
   - Option C: Separate GlobalEvent handlers that run before/after spatial phases

3. **Event priority**: Should some events run before others within same color?
   - evenio has High/Medium/Low priority
   - Minecraft might need: Physics → Collision → Damage → Death order

4. **Chunk loading**: Chunks aren't entities with Position. How do they fit?
   - Option A: Chunks are global, loaded in Phase 0
   - Option B: Chunks have pseudo-position at center, participate in RGB
   - Option C: Chunk operations are always global events

## Implementation Phases

### Phase 1: Event Infrastructure
- [ ] Define `Event`, `GlobalEvent`, `SpatialEvent`, `TargetedEvent` traits
- [ ] Implement `EventQueue` with RGB bucketing
- [ ] Create `Handler` trait and registration
- [ ] Basic `Receiver<E>` handler parameter

### Phase 2: Handler Parameters
- [ ] `Fetcher<Q>` for component access
- [ ] `Single<Q>` for global/unique components
- [ ] `With<C>` / `Without<C>` filters
- [ ] `Sender<E>` for emitting events from handlers

### Phase 3: Spatial Integration
- [ ] `Neighbors<Q>` for accessing adjacent cells
- [ ] RGB phase execution with barriers
- [ ] Parallel execution within color phases

### Phase 4: RPC Macro
- [ ] `#[rpc]` proc macro for packet → event
- [ ] Auto-generate event structs
- [ ] Packet dispatch table generation

### Phase 5: Migration
- [ ] Convert existing systems to handlers
- [ ] Replace manual packet parsing with RPC handlers
- [ ] Add audio profiling hooks per phase

## Example: Full Movement Flow

```rust
// 1. Network layer receives packet 0x1D (MovePlayerPos)
// 2. Packet decoder creates MoveEvent, queues to entity's color bucket

#[rpc(MovePlayerPos = 0x1D)]
fn decode_move(x: f64, y: f64, z: f64) -> MoveEvent {
    MoveEvent { x, y, z }
}

// 3. During entity's color phase, handler runs

#[handler]
fn handle_movement(
    event: Receiver<MoveEvent>,
    mut positions: Fetcher<&mut Position>,
    grid: Single<&SpatialGrid>,
) {
    let entity = event.target();
    let mut pos = positions.get_mut(entity).unwrap();

    // Update position (may change cell assignment)
    pos.update(&grid, event.x, event.y, event.z);
}

// 4. Collision handler also triggers for entities with Collider

#[handler]
fn check_collision(
    event: Receiver<MoveEvent>,
    _: With<Collider>,
    positions: Fetcher<&Position>,
    neighbors: Neighbors<(&Position, &Hitbox)>,
    sender: Sender<CollisionEvent>,
) {
    let entity = event.target();
    let pos = positions.get(entity).unwrap();

    for (other, (other_pos, hitbox)) in neighbors.iter() {
        if hitbox.intersects(pos, other_pos) {
            sender.send_to(entity, CollisionEvent { other });
        }
    }
}

// 5. CollisionEvent triggers in same color phase (queued behind current event)

#[handler]
fn handle_collision(
    event: Receiver<CollisionEvent>,
    mut velocities: Fetcher<&mut Velocity>,
) {
    // Bounce off collision
    let mut vel = velocities.get_mut(event.target()).unwrap();
    vel.reverse();
}
```

## Example: Player Combat (Spatial Event)

```rust
// Packet 0x16: Player attacks entity
#[rpc(InteractEntity = 0x16)]
fn decode_attack(target_id: VarInt, action: VarInt) -> Option<AttackEvent> {
    if action.0 == 1 { // Attack action
        Some(AttackEvent { target_entity_id: target_id.0 })
    } else {
        None
    }
}

#[derive(SpatialEvent)]
struct AttackEvent {
    target_entity_id: i32,
}

#[derive(SpatialEvent)]
struct DamageEvent {
    amount: f32,
    attacker: Entity,
}

// Handler 1: Validate attack is in range (same/adjacent cell)
#[handler]
fn validate_attack(
    event: Receiver<AttackEvent>,
    attacker_pos: Fetcher<&Position>,
    neighbors: Neighbors<(&Position, &EntityId)>,  // Can see adjacent cells
    sender: Sender<DamageEvent>,
) {
    let attacker = event.target();
    let attacker_p = attacker_pos.get(attacker).unwrap();

    // Find target entity in our cell or neighbors
    for (target_entity, (target_pos, eid)) in neighbors.iter() {
        if eid.value == event.target_entity_id {
            // Check within attack range (3 blocks)
            let dist_sq = (attacker_p.x - target_pos.x).powi(2)
                        + (attacker_p.z - target_pos.z).powi(2);
            if dist_sq <= 9.0 {
                // Valid attack - send damage event to target
                sender.send_to(target_entity, DamageEvent {
                    amount: 1.0,
                    attacker,
                });
            }
            return;
        }
    }
    // Target not found in range - invalid attack (ignored or log)
}

// Handler 2: Apply damage (runs on target entity's color phase)
#[handler]
fn apply_damage(
    event: Receiver<DamageEvent>,
    mut health: Fetcher<&mut Health>,
    sender: Sender<DeathEvent>,
) {
    let target = event.target();
    let mut hp = health.get_mut(target).unwrap();

    hp.current -= event.amount;

    if hp.current <= 0.0 {
        sender.send_to(target, DeathEvent { killer: Some(event.attacker) });
    }
}

// Handler 3: Handle death
#[handler]
fn handle_death(
    event: Receiver<DeathEvent>,
    positions: Fetcher<&Position>,
    sender: Sender<RespawnEvent>,
) {
    let entity = event.target();
    // Queue respawn, drop items, etc.
    sender.send_to(entity, RespawnEvent { position: Position::SPAWN });
}
```

**Key insight**: `Neighbors<Q>` lets spatial handlers read entities in adjacent cells (different colors), but can only **write** to entities in the same cell (same color). This is what makes RGB parallelism safe:

- RED cell handler can **read** GREEN/BLUE neighbors
- RED cell handler can only **write** to RED entities
- No write conflicts between parallel RED cells (they don't share edges)

When AttackEvent targets an entity in a different-colored cell, the DamageEvent is queued to that entity's color bucket and processed in the appropriate phase.

## Alternatives Considered

### 1. Keep Systems, Add Parallelism
Run existing systems but partition by cell color. Simpler but less composable.

### 2. Full evenio Port
Use evenio directly without RGB modifications. Loses spatial parallelism.

### 3. Actor Model
Each cell is an actor with message passing. More isolation but higher latency.

### 4. ECS + Scripting
Core in Rust ECS, gameplay in Lua/WASM. Flexibility but performance overhead.

## Recommendation

Start with **Phase 1-2** to get event infrastructure working with existing sequential execution. This validates the API design without the complexity of parallelism.

Then add **Phase 3** RGB parallelism as an optimization layer. The handler code shouldn't need to change - only the execution scheduler.

The `#[rpc]` macro (**Phase 4**) can be added incrementally as packets are converted.
