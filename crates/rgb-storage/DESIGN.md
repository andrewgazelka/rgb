# RGB Storage Design: Versioned Append-Only B-Tree

## Requirements

1. **Full history**: Revert to ANY tick, not just periodic snapshots
2. **Efficient writes**: Only log changed data per tick
3. **Efficient reads**: Current tick should be fast
4. **Time-travel queries**: Read state at any historical tick
5. **Space efficient**: Share unchanged data across ticks

## Architecture: Copy-on-Write B+Tree

Inspired by [Nebari](https://github.com/khonsulabs/nebari) and MVCC B-trees.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        TICK ROOTS                                   │
│                                                                     │
│  Tick 0 ──► Root₀                                                   │
│  Tick 1 ──► Root₁ ──────────────────┐                              │
│  Tick 2 ──► Root₂ ─────────┐        │                              │
│  Tick 3 ──► Root₃          │        │                              │
│             │              │        │                              │
│             ▼              ▼        ▼                              │
│         ┌──────┐       ┌──────┐  ┌──────┐                          │
│         │Page A│       │Page B│  │Page C│  ← Shared across ticks   │
│         │(new) │       │(tick1)│ │(tick0)│                         │
│         └──────┘       └──────┘  └──────┘                          │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Insight: Copy-on-Write

When a component changes at tick N:
1. Copy the path from root to the leaf containing the data
2. Only new/modified pages are written
3. Unchanged pages are shared with previous ticks
4. Previous tick's root still points to old data

### On-Disk Format

```
┌─────────────────────────────────────────────────────────────────────┐
│  FILE: world.rgb                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  [Header]                                                            │
│    magic: "RGB\0"                                                    │
│    version: u32                                                      │
│    page_size: u32 (4KB default)                                     │
│    tick_index_offset: u64  (points to tick index at end of file)    │
├─────────────────────────────────────────────────────────────────────┤
│  [Page 0] Root for tick 0                                           │
│  [Page 1] Leaf node                                                  │
│  [Page 2] Leaf node                                                  │
│  ...                                                                 │
│  [Page N] Root for tick 1 (points to some existing + new pages)     │
│  [Page N+1] New leaf (only changed data)                            │
│  ...                                                                 │
├─────────────────────────────────────────────────────────────────────┤
│  [Tick Index] (append-only, at end of file)                         │
│    tick 0: root_page=0, timestamp=..., entity_count=...             │
│    tick 1: root_page=N, timestamp=..., entity_count=...             │
│    ...                                                               │
│  [Footer]                                                            │
│    tick_count: u64                                                   │
│    checksum: u64                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Page Format

```rust
/// 4KB page (configurable)
#[repr(C)]
struct Page {
    header: PageHeader,  // 16 bytes
    data: [u8; PAGE_SIZE - 16],
}

#[repr(C)]
struct PageHeader {
    page_type: PageType,  // u8: Internal, Leaf, Overflow
    flags: u8,
    entry_count: u16,
    tick_created: u32,    // Which tick created this page
    checksum: u64,
}

enum PageType {
    Internal = 0,  // B+tree internal node
    Leaf = 1,      // B+tree leaf (contains component data)
    Overflow = 2,  // Large values that don't fit in leaf
}
```

### B+Tree Structure

```rust
/// Internal node: keys + child page references
struct InternalNode {
    // Sorted keys (entity_id, component_type_id)
    keys: Vec<(EntityId, ComponentId)>,
    // Child page offsets (one more than keys)
    children: Vec<PageOffset>,
}

/// Leaf node: actual component data
struct LeafNode {
    entries: Vec<LeafEntry>,
    next_leaf: Option<PageOffset>,  // For range scans
}

struct LeafEntry {
    entity_id: EntityId,
    component_id: ComponentId,
    // Inline if small, overflow page reference if large
    value: ComponentValue,
}

enum ComponentValue {
    Inline(Vec<u8>),           // Small values inline
    Overflow(PageOffset),      // Large values in overflow pages
}
```

## Operations

### Write (on tick commit)

```rust
impl VersionedTree {
    /// Commit all changes for a tick, returns new root
    pub fn commit_tick(&mut self, tick: TickId, mutations: &[Mutation]) -> PageOffset {
        // Start with current root
        let mut new_root = self.current_root.clone();

        for mutation in mutations {
            match mutation {
                Mutation::Insert { entity, component, value } => {
                    // Copy-on-write path from root to leaf
                    new_root = self.insert_cow(new_root, entity, component, value);
                }
                Mutation::Update { entity, component, value } => {
                    new_root = self.update_cow(new_root, entity, component, value);
                }
                Mutation::Remove { entity, component } => {
                    new_root = self.remove_cow(new_root, entity, component);
                }
            }
        }

        // Append new root page to file
        let root_offset = self.append_page(new_root);

        // Record in tick index
        self.tick_index.push(TickEntry {
            tick,
            root_page: root_offset,
            timestamp: now(),
        });

        root_offset
    }
}
```

### Read (current tick)

```rust
impl VersionedTree {
    /// Get component at current tick (fast path)
    pub fn get<T: Component>(&self, entity: Entity) -> Option<T> {
        self.get_at_tick(entity, self.current_tick)
    }
}
```

### Time Travel Read

```rust
impl VersionedTree {
    /// Get component at any historical tick
    pub fn get_at_tick<T: Component>(&self, entity: Entity, tick: TickId) -> Option<T> {
        // Look up root for that tick
        let tick_entry = self.tick_index.get(tick)?;
        let root = self.read_page(tick_entry.root_page);

        // Standard B+tree lookup from that root
        self.btree_get(root, entity, T::component_id())
    }

    /// Get entire world state at a tick (for replay/debug)
    pub fn snapshot_at_tick(&self, tick: TickId) -> WorldSnapshot {
        let tick_entry = self.tick_index.get(tick)?;
        WorldSnapshot::from_root(self, tick_entry.root_page)
    }
}
```

### Revert to Tick

```rust
impl VersionedTree {
    /// Revert world state to a previous tick
    ///
    /// This doesn't delete history - future ticks are still accessible.
    /// It just changes what "current" points to.
    pub fn revert_to_tick(&mut self, tick: TickId) -> Result<()> {
        let tick_entry = self.tick_index.get(tick)?;
        self.current_tick = tick;
        self.current_root = tick_entry.root_page;
        Ok(())
    }

    /// Revert and TRUNCATE (delete all ticks after)
    pub fn revert_and_truncate(&mut self, tick: TickId) -> Result<()> {
        self.revert_to_tick(tick)?;
        self.tick_index.truncate(tick + 1);
        // Pages from truncated ticks become garbage
        // Can be reclaimed by compaction
        Ok(())
    }
}
```

## Space Efficiency

### Structural Sharing

With copy-on-write, unchanged data is shared:

```
Tick 0: 1000 entities, 10 components each = ~10K entries
Tick 1: 50 entities changed = ~500 new entries, 9.5K shared
Tick 2: 100 entities changed = ~1K new entries, rest shared

Storage: tick 0 = 10K entries
         tick 1 = 500 new + pointers to shared = minimal overhead
         tick 2 = 1K new + pointers to shared = minimal overhead
```

### Compaction (optional)

For long-running servers, can compact old ticks:

```rust
impl VersionedTree {
    /// Compact history, keeping only ticks at intervals
    /// Example: keep every 1000th tick before tick N
    pub fn compact(&mut self, before_tick: TickId, keep_interval: u32) {
        // Rewrite file with only kept ticks
        // Reclaim pages only referenced by removed ticks
    }
}
```

## Integration with ECS

```rust
/// The World now wraps a VersionedTree
pub struct World {
    tree: VersionedTree,
    // In-memory caches for hot path
    entity_cache: EntityCache,
    // Pending mutations (batched until tick commit)
    pending: Vec<Mutation>,
}

impl World {
    /// Get component (reads from tree or cache)
    pub fn get<T: Component + Clone>(&self, entity: Entity) -> Option<T> {
        // Check pending mutations first
        if let Some(value) = self.pending.get_pending(entity, T::ID) {
            return Some(value.clone());
        }
        // Then tree
        self.tree.get::<T>(entity)
    }

    /// Update component (buffers until tick commit)
    pub fn update<T: Component>(&mut self, entity: Entity, value: T) {
        self.pending.push(Mutation::Update {
            entity,
            component: T::ID,
            value: value.serialize(),
        });
    }

    /// Commit all pending mutations atomically
    pub fn commit_tick(&mut self) -> TickId {
        let tick = self.tree.current_tick + 1;
        self.tree.commit_tick(tick, &self.pending);
        self.pending.clear();
        tick
    }
}
```

## Comparison with Alternatives

| Approach | Revert to any tick | Space efficiency | Write speed | Read speed |
|----------|-------------------|------------------|-------------|------------|
| **Versioned B+tree (this)** | ✅ Yes | ✅ Excellent (sharing) | ✅ Good | ✅ Excellent |
| WAL + periodic snapshots | ❌ Only snapshots fast | ⚠️ WAL grows | ✅ Fast append | ✅ Fast current |
| Full snapshot per tick | ✅ Yes | ❌ Huge | ❌ Slow (full copy) | ✅ Fast |
| Event sourcing only | ✅ Yes (via replay) | ✅ Minimal | ✅ Fast append | ❌ Slow (replay) |

## Implementation Dependencies

Consider using:
- [Nebari](https://github.com/khonsulabs/nebari) - Rust append-only B-tree (could use directly)
- Custom implementation for maximum control over page format

## Next Steps

1. Implement basic versioned B+tree with CoW
2. Add page caching (mmap or explicit cache)
3. Add tick index with binary search
4. Add compaction for long-running servers
5. Add checksums and recovery
