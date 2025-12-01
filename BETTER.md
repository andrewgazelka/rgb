# What Flecs and Evenio Do Better Than RGB

A comparison of elegant patterns from [flecs](https://github.com/SanderMertens/flecs) (C/C++ ECS) and [evenio](https://github.com/rj00a/evenio) (Rust ECS) that RGB could adopt.

---

## 1. Query Combinators and Type-Level DSL (Evenio)

**Evenio's elegant approach:**
```rust
// Compose queries with type-level operators
Fetcher<(&Position, &mut Velocity)>           // AND
Fetcher<Or<&Player, &Enemy>>                   // OR (either or both)
Fetcher<Xor<&Flying, &Grounded>>               // XOR (exactly one)
Fetcher<Option<&Health>>                       // Optional component
Fetcher<Not<&Dead>>                            // Exclusion filter
Fetcher<With<&Marker>>                         // Filter without access
Fetcher<Has<&Tag>>                             // Boolean check (no access cost)
```

**RGB's current approach:**
```rust
// Limited to single-component iteration
world.query::<Position>()
// No composition, no filters, no optional access
```

**Why it's better:** Evenio's query system is a compile-time DSL that:
- Prevents aliased mutability at compile time via DNF analysis
- Allows complex filters without runtime overhead
- Supports `#[derive(Query)]` for named field access

---

## 2. Handler/System Parameter Injection (Evenio)

**Evenio's approach:**
```rust
fn my_handler(
    _: Receiver<DamageEvent>,           // Event receiver (required)
    entities: Fetcher<(&Health, &Armor)>, // Query all matching entities
    player: Single<&mut Player>,        // Singleton (panics if != 1)
    config: TrySingle<&Config>,         // Fallible singleton
    sender: Sender<DeathEvent>,         // Event sender
    mut counter: Local<u32>,            // Handler-local state
    handlers: &Handlers,                // Metadata introspection
) { ... }
```

**RGB's current approach:**
```rust
// Manual world access, no automatic injection
fn update(world: &mut World) {
    for (entity, pos) in world.query::<Position>() {
        let vel = world.get::<Velocity>(entity);
        // Manual plumbing everywhere
    }
}
```

**Why it's better:**
- Zero boilerplate for common patterns
- `Local<T>` provides handler-scoped state without globals
- `Single<T>` / `TrySingle<T>` for singleton access patterns
- Automatic conflict detection at handler registration time

---

## 3. First-Class Entity Relationships (Flecs)

**Flecs's approach:**
```cpp
// Relationships are first-class pairs: (Relationship, Target)
entity.add<ChildOf>(parent);
entity.add<InstanceOf>(prefab);
entity.add(Eats, Apples);  // Custom relationships

// Query relationships
world.query<Position>().with<ChildOf>(parent).each(...);

// Transitive traversal
world.query<Position>().with<ChildOf>().cascade().each(...);

// Wildcard queries
world.query<>().with(Likes, flecs::Wildcard).each(...);
```

**RGB's current approach:**
```rust
// Relationships stored as regular components (Pair<R>)
world.insert_pair::<ChildOf>(child, parent);
// No transitive traversal
// No wildcard queries
// No relationship caching
```

**Why it's better:**
- Flecs has dedicated pair indexing for O(1) relationship lookups
- Transitive relationship support (walk up hierarchy automatically)
- Relationship caching for computed traversals
- Query DSL supports relationship patterns

---

## 4. Fluent Entity Builder API (Flecs)

**Flecs's approach:**
```cpp
auto entity = world.entity("Player")
    .set<Position>({10, 20})
    .set<Velocity>({1, 2})
    .add<Player>()
    .add<ChildOf>(parent)
    .insert([](Position& p, Velocity& v) {
        p = {0, 0};
        v = {1, 1};
    });
```

**RGB's current approach:**
```rust
let entity = world.spawn_empty();
world.insert(entity, Position { x: 10.0, y: 20.0 });
world.insert(entity, Velocity { x: 1.0, y: 2.0 });
// Multiple calls, no chaining
```

**Why it's better:**
- Single expression creates fully-configured entity
- Lambda-based initialization for complex setup
- Method chaining reduces boilerplate

---

## 5. Component Lifecycle Hooks (Flecs)

**Flecs's approach:**
```cpp
world.component<Position>()
    .on_add([](Position& p) { /* initialization */ })
    .on_set([](Position& p) { /* value changed */ })
    .on_remove([](Position& p) { /* cleanup */ });

// Also supports: ctor, dtor, copy, move
```

**RGB's current approach:**
```rust
// No lifecycle hooks
// Must manually track changes via observers/events
```

**Why it's better:**
- Automatic resource management tied to component lifecycle
- Change detection built into the ECS
- Initialization logic colocated with component definition

---

## 6. Compile-Time Access Conflict Detection (Evenio)

**Evenio's approach:**
```rust
// This FAILS at handler registration:
world.add_handler(|_: Receiver<E>, _: Fetcher<(&mut A, &mut A)>| {});
// Error: component A accessed mutably twice

// This SUCCEEDS (mutual exclusion):
world.add_handler(|_: Receiver<E>, _: Fetcher<Xor<&mut A, &mut B>>| {});
```

**RGB's current approach:**
```rust
// No compile-time or registration-time checks
// Runtime panics or undefined behavior possible
```

**Why it's better:**
- Impossible to write handlers with aliasing violations
- Uses Disjunctive Normal Form (DNF) for complex access analysis
- Errors at handler registration, not at runtime

---

## 7. Event Priority and Ordering (Evenio)

**Evenio's approach:**
```rust
world.add_handler(damage_handler.high());    // Runs first
world.add_handler(death_handler);            // Default priority
world.add_handler(cleanup_handler.low());   // Runs last

// Same priority: insertion order tiebreaker
```

**RGB's current approach:**
```rust
// Observers have no priority system
// Order is undefined
```

**Why it's better:**
- Explicit control over handler execution order
- Three-tier system (High/Medium/Low) is intuitive
- Insertion order provides deterministic fallback

---

## 8. Immutable Components (Evenio)

**Evenio's approach:**
```rust
#[derive(Component)]
#[component(immutable)]
struct EntityId(u64);

// Compile error: can't get &mut EntityId
// Must use events to modify immutable components
```

**RGB's current approach:**
```rust
// All components are mutable
// No way to enforce read-only semantics
```

**Why it's better:**
- Enforces invariants at the type level
- Useful for IDs, configurations, and security-sensitive data
- Forces changes through controlled event channels

---

## 9. Query Result Caching (Flecs)

**Flecs's approach:**
```cpp
// Query is compiled once, cached
auto q = world.query<Position, Velocity>();

// Iteration uses cached archetype pointers
q.each([](Position& p, Velocity& v) { ... });

// Cache automatically invalidated when archetypes change
```

**RGB's current approach:**
```rust
// Fresh iteration every time
for (entity, pos) in world.query::<Position>() {
    // Re-filters archetypes each call
}
```

**Why it's better:**
- Amortized O(1) iteration after initial query
- Bloom filters for quick archetype rejection
- Automatic cache invalidation via generation numbers

---

## 10. Reflection and Serialization (Flecs)

**Flecs's approach:**
```cpp
// Components are automatically serializable
world.component<Position>()
    .member<float>("x")
    .member<float>("y");

// JSON serialization built-in
std::string json = entity.to_json();
entity.from_json(json);

// Runtime introspection
for (auto& member : type.members()) { ... }
```

**RGB's current approach:**
```rust
// No reflection system
// Manual serde implementation required
// No runtime type introspection
```

**Why it's better:**
- Debugging, tooling, and save/load all benefit
- Query by component name (string) at runtime
- Remote API for live introspection

---

## 11. Targeted Events (Evenio)

**Evenio's approach:**
```rust
#[derive(TargetedEvent)]
struct Damage(u32);

// Send to specific entity
world.send_to(enemy, Damage(10));

// Handler automatically receives target entity
fn on_damage(r: Receiver<Damage, &mut Health>) {
    r.query.0 -= r.event.0;  // query = target's Health
}
```

**RGB's current approach:**
```rust
// Events have Target component, but observer dispatch
// doesn't automatically filter by target
// Manual target resolution required
```

**Why it's better:**
- Events are routed to relevant handlers only
- Target entity's components are automatically in scope
- No wasted work processing events for wrong entities

---

## 12. `#[derive(Query)]` for Named Fields (Evenio)

**Evenio's approach:**
```rust
#[derive(Query)]
struct MovementQuery<'a> {
    pos: &'a mut Position,
    vel: &'a Velocity,
    id: EntityId,
    flying: Has<&'static Flying>,
}

fn movement_system(_: Receiver<Tick>, q: Fetcher<MovementQuery>) {
    for m in q {
        m.pos.x += m.vel.x;  // Named field access!
        if m.flying.get() { ... }
    }
}
```

**RGB's current approach:**
```rust
// Only tuple access
for (entity, pos) in world.query::<Position>() { ... }
```

**Why it's better:**
- Self-documenting queries with named fields
- Complex queries remain readable
- Automatic lifetime handling via proc macro

---

## 13. Parallel Iteration (Evenio + Flecs)

**Evenio's approach:**
```rust
#[cfg(feature = "rayon")]
for item in fetcher.par_iter() { ... }
```

**Flecs's approach:**
```cpp
system.multi_threaded()  // Automatic parallelization
```

**RGB's current approach:**
```rust
// RGB parallelism is per-chunk (RGB coloring)
// But no parallel iteration within a system
// Deferred mutations only
```

**Observation:** RGB's spatial parallelism is novel, but per-system parallel iteration is missing.

---

## 14. Entity References (Flecs)

**Flecs's approach:**
```cpp
// Fast, cached component access
flecs::ref<Position> pos_ref = entity.get_ref<Position>();

// Stays valid across archetype changes (with version checking)
Position& p = *pos_ref;  // O(1) access
```

**RGB's current approach:**
```rust
// Must re-lookup each time
let pos = world.get::<Position>(entity);
// Clone required, no caching
```

**Why it's better:**
- Avoids repeated hash lookups
- Version-checked for safety
- Critical for hot paths

---

## 15. Pipeline and Schedule System (Flecs)

**Flecs's approach:**
```cpp
// Automatic dependency analysis
world.system<Position, Velocity>("Move").kind<OnUpdate>();
world.system<Position>("Render").kind<OnStore>().after<Move>();

// Pipeline phases
PreUpdate -> OnUpdate -> OnValidate -> PostUpdate -> OnStore
```

**RGB's current approach:**
```rust
// Manual tick phases (7 phases in rgb-tick)
// No automatic dependency analysis
// No declarative system ordering
```

**Why it's better:**
- Declarative system ordering with dependency inference
- Pipeline phases for frame lifecycle
- Automatic parallelization of non-conflicting systems

---

## Summary: Priority Improvements for RGB

| Priority | Feature | Source | Effort |
|----------|---------|--------|--------|
| High | Query combinators (`Or`, `Not`, `Option`, etc.) | Evenio | Medium |
| High | Handler parameter injection | Evenio | High |
| High | Compile-time access conflict detection | Evenio | High |
| Medium | Entity builder fluent API | Flecs | Low |
| Medium | Event priority system | Evenio | Low |
| Medium | `#[derive(Query)]` for named fields | Evenio | Medium |
| Medium | Component lifecycle hooks | Flecs | Medium |
| Low | Immutable components | Evenio | Low |
| Low | Query caching | Flecs | Medium |
| Low | Reflection/serialization | Flecs | High |

---

## Notes

- RGB's unique strength is **spatial RGB parallelism** which neither Flecs nor Evenio have
- RGB's **versioned B+tree storage** for tick history is also unique
- The **owned-value API** (get/update pattern) is a deliberate design choice for hot-reload

The goal isn't to copy everything, but to identify ergonomic and safety patterns that would enhance RGB without sacrificing its spatial-first architecture.
