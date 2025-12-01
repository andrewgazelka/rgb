# RGB ECS Next Steps

## Immediate Fixes

### 1. Update mc-server-runner components to use derive ✅ DONE
All components in mc-server-runner now use `#[derive(Component)]`:
- POD components (simple flat data): Position, Rotation, EntityId, etc.
- Opaque components (`#[component(opaque)]`): PacketBuffer, Name, NetworkIngress, etc.

### 2. Better Clone error message
Detect missing Clone in the derive and emit a clear error:
```
error: Component requires Clone. Add #[derive(Clone)] to your type.
```

### 3. Document opaque in CLAUDE.md ✅ DONE
Added section explaining POD vs opaque components, allowed types (Bytes), and NO LOCKS rule.

---

## Relational Refactor (Medium Priority)

### Goal: Replace VecDeque/collection components with relations

**Current (bad):**
```rust
#[component(opaque)]
struct PacketBuffer {
    incoming: VecDeque<(i32, Bytes)>,  // Clone entire queue every access
}
```

**Target (good):**
```rust
// Each packet is its own entity
#[derive(Component, Clone)]
struct PendingPacket {
    packet_id: i32,
    sequence: u64,  // For ordering
}

// Relation to connection
world.spawn((
    PendingPacket { packet_id: 0x10, sequence: 1 },
    Pair::<PendingFor>(connection_entity),
));

// Large data in side-table resource
world.resource_mut::<PacketDataStore>().insert(packet_entity, bytes);
```

**Benefits:**
- No cloning collections
- Natural ordering via `order_by`
- Parallel-safe (different connections = different entities)
- Queryable

### Steps:
1. Add `Resource` system to World (global singletons, any type)
2. Add pair queries to QueryBuilder: `.pair::<Relation>(target)`
3. Add `group_by` to queries (group by relation target)
4. Refactor `PacketBuffer` → packet entities with relations
5. Refactor `PendingPackets` → same pattern

---

## Query Enhancements (Medium Priority)

### 1. Pair queries
```rust
let query = world.query()
    .with::<PacketData>()
    .pair::<PendingFor>(connection)  // Filter by relation target
    .build();
```

### 2. Wildcard pairs
```rust
let query = world.query()
    .with::<PacketData>()
    .pair::<PendingFor>(Wildcard)  // All pending packets, any connection
    .build();
```

### 3. Group by relation
```rust
let query = world.query()
    .with::<PacketData>()
    .group_by::<PendingFor>()  // Group results by connection
    .build();

for (connection, packets) in query.iter_groups(&world) {
    // Process all packets for this connection together
}
```

### 4. Order by component
```rust
let query = world.query()
    .with::<PacketData>()
    .order_by::<Sequence>()  // Sort by sequence number
    .build();
```

---

## Resource System - NOT NEEDED

~~Global singletons~~ → Use opaque components on `Entity::WORLD` instead:

```rust
// Store global state on WORLD entity with opaque components
world.update(Entity::WORLD, NetworkIngress { rx });
world.update(Entity::WORLD, PacketDataStore::new());

// Access like any other component
let network = world.get::<NetworkIngress>(Entity::WORLD)?;
```

This is simpler and more consistent - everything is entities + components.

---

## Enforcement Options (Low Priority, Discuss First)

### Option A: Keep advisory (current)
- Derive is optional but recommended
- Flexible, like serde
- Users can bypass checks

### Option B: Soft enforcement
- Add `#[allow(non_pod_component)]` for explicit opt-out
- Clippy-style warnings if derive not used

### Option C: Hard enforcement
- Remove blanket impl
- World methods require `T: ValidatedComponent`
- Only derive provides this trait
- Breaking change

**Recommendation:** Stay with Option A for now. The derive provides value through validation and documentation, but forcing it adds friction without much benefit.

---

## Priority Order

1. ~~**Resource system**~~ - NOT NEEDED, use opaque components on Entity::WORLD
2. ~~**Update mc-server-runner components**~~ ✅ DONE
3. **Pair queries** - Enables relational patterns
4. **Relational refactor of PacketBuffer** - Proves the pattern works
5. **Group by / order by** - Polish

---

## Non-Goals (For Now)

- Hot reloading (blocked on Rust ABI stability)
- Scripting integration (Lua/WASM)
- Full persistence (needs more design)
- Change detection / dirty tracking
