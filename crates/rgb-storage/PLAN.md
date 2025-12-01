# rgb-storage Plan

## Purpose

Persistent storage layer with Write-Ahead Log (WAL) for durability, snapshots for fast recovery, and time-travel queries for debugging/replay.

## Core Concepts

### Tick-Based Persistence

Every tick produces a deterministic set of mutations. These are logged sequentially:

```
WAL Structure:
┌─────────┬─────────┬─────────┬─────────┬─────────┐
│ Tick 0  │ Tick 1  │ Tick 2  │ Tick 3  │ Tick 4  │
│ Genesis │ Δ₁      │ Δ₂      │ Δ₃      │ Δ₄      │
└─────────┴─────────┴─────────┴─────────┴─────────┘

Snapshots (periodic):
┌─────────────────────────────────────────────────┐
│ Snapshot @ Tick 100 (full state)                │
└─────────────────────────────────────────────────┘
```

### Recovery Process

1. Load latest snapshot
2. Replay WAL entries after snapshot tick
3. Resume from recovered state

## Core Types

### `StorageEngine`

```rust
pub struct StorageEngine {
    /// WAL for tick deltas
    wal: WriteAheadLog,

    /// Snapshot manager
    snapshots: SnapshotManager,

    /// Storage backend
    backend: Box<dyn StorageBackend>,

    /// Current committed tick
    committed_tick: u64,
}

impl StorageEngine {
    /// Open or create storage at path
    pub fn open(path: &Path, config: StorageConfig) -> eyre::Result<Self>;

    /// Commit a tick's mutations
    pub fn commit_tick(&mut self, tick: u64, mutations: &TickMutations) -> eyre::Result<()>;

    /// Create a snapshot at current tick
    pub fn create_snapshot(&mut self, world: &World) -> eyre::Result<SnapshotId>;

    /// Recover world state to a specific tick
    pub fn recover(&self, target_tick: u64) -> eyre::Result<World>;

    /// Get the latest committed tick
    pub fn latest_tick(&self) -> u64;
}
```

### `WriteAheadLog`

```rust
pub struct WriteAheadLog {
    /// Active segment file
    active_segment: SegmentFile,

    /// Segment rotation config
    segment_size: u64,

    /// Index for tick lookups
    tick_index: TickIndex,
}

impl WriteAheadLog {
    /// Append a tick entry
    pub fn append(&mut self, entry: WalEntry) -> eyre::Result<()>;

    /// Read entries in range
    pub fn read_range(&self, start_tick: u64, end_tick: u64) -> eyre::Result<Vec<WalEntry>>;

    /// Truncate WAL before tick (after snapshot)
    pub fn truncate_before(&mut self, tick: u64) -> eyre::Result<()>;

    /// Sync to disk
    pub fn sync(&mut self) -> eyre::Result<()>;
}
```

### `WalEntry`

```rust
/// Single tick's changes
pub struct WalEntry {
    /// Tick number
    pub tick: u64,

    /// Checksum for integrity
    pub checksum: u32,

    /// Serialized mutations
    pub mutations: TickMutations,
}

/// All mutations from a tick
pub struct TickMutations {
    /// Component insertions
    pub inserts: Vec<InsertMutation>,

    /// Component removals
    pub removals: Vec<RemovalMutation>,

    /// Entity spawns
    pub spawns: Vec<SpawnMutation>,

    /// Entity despawns
    pub despawns: Vec<Entity>,

    /// Singleton updates
    pub singleton_updates: Vec<SingletonMutation>,
}
```

### `SnapshotManager`

```rust
pub struct SnapshotManager {
    /// Snapshot storage directory
    snapshot_dir: PathBuf,

    /// Available snapshots
    snapshots: Vec<SnapshotMeta>,

    /// Snapshot policy
    policy: SnapshotPolicy,
}

pub struct SnapshotMeta {
    /// Unique identifier
    pub id: SnapshotId,

    /// Tick this snapshot represents
    pub tick: u64,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Snapshot size in bytes
    pub size: u64,
}

pub enum SnapshotPolicy {
    /// Snapshot every N ticks
    EveryNTicks(u64),

    /// Snapshot when WAL exceeds size
    WalSizeThreshold(u64),

    /// Manual snapshots only
    Manual,
}
```

### `Snapshot`

```rust
/// Full world state at a point in time
pub struct Snapshot {
    /// Tick number
    pub tick: u64,

    /// All entities and their components
    pub entities: Vec<EntitySnapshot>,

    /// All singletons
    pub singletons: Vec<SingletonSnapshot>,

    /// Spatial grid state
    pub spatial: SpatialSnapshot,
}

pub struct EntitySnapshot {
    pub entity: Entity,
    pub chunk: ChunkId,
    pub components: Vec<(ComponentId, Box<[u8]>)>,
}
```

## Storage Backend Trait

```rust
/// Pluggable storage backend
pub trait StorageBackend: Send + Sync {
    /// Write bytes at key
    fn put(&mut self, key: &[u8], value: &[u8]) -> eyre::Result<()>;

    /// Read bytes at key
    fn get(&self, key: &[u8]) -> eyre::Result<Option<Vec<u8>>>;

    /// Delete key
    fn delete(&mut self, key: &[u8]) -> eyre::Result<()>;

    /// Iterate over key range
    fn range(&self, start: &[u8], end: &[u8]) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>;

    /// Sync to durable storage
    fn sync(&mut self) -> eyre::Result<()>;
}

/// Simple file-based backend
pub struct FileBackend { ... }

/// Memory backend for testing
pub struct MemoryBackend { ... }

/// LMDB-style backend (optional feature)
#[cfg(feature = "lmdb")]
pub struct LmdbBackend { ... }
```

## Time-Travel API

```rust
impl StorageEngine {
    /// Query world state at historical tick
    pub fn at_tick(&self, tick: u64) -> eyre::Result<HistoricalWorld>;

    /// Get entity's component history
    pub fn entity_history(
        &self,
        entity: Entity,
        component: ComponentId,
        tick_range: Range<u64>,
    ) -> eyre::Result<Vec<(u64, Option<Box<[u8]>>)>>;

    /// Find tick where condition became true
    pub fn find_tick<F>(&self, range: Range<u64>, predicate: F) -> eyre::Result<Option<u64>>
    where
        F: Fn(&HistoricalWorld) -> bool;
}

/// Read-only view of world at a past tick
pub struct HistoricalWorld {
    tick: u64,
    // Lazily reconstructed state
    state: OnceCell<World>,
    storage: Arc<StorageEngine>,
}

impl HistoricalWorld {
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T>;
    pub fn singleton<T: Singleton>(&self) -> Option<&T>;
    pub fn query<Q: QueryData>(&self) -> HistoricalQuery<Q>;
}
```

## WAL Format

Binary format for efficiency:

```
┌────────────────────────────────────────────────┐
│ WAL Segment Header (64 bytes)                   │
│  - Magic: "RGBWAL" (6 bytes)                   │
│  - Version: u16                                 │
│  - Segment ID: u64                              │
│  - Start Tick: u64                              │
│  - Reserved: 42 bytes                           │
├────────────────────────────────────────────────┤
│ Entry 1                                         │
│  - Length: u32                                  │
│  - Tick: u64                                    │
│  - Checksum: u32                                │
│  - Mutations: [bytes]                           │
├────────────────────────────────────────────────┤
│ Entry 2                                         │
│  ...                                            │
└────────────────────────────────────────────────┘
```

## Snapshot Format

```
┌────────────────────────────────────────────────┐
│ Snapshot Header                                 │
│  - Magic: "RGBSNAP" (7 bytes)                  │
│  - Version: u16                                 │
│  - Tick: u64                                    │
│  - Entity Count: u64                            │
│  - Singleton Count: u32                         │
│  - Checksum: u32                                │
├────────────────────────────────────────────────┤
│ Entity Table                                    │
│  - [EntitySnapshot serialized...]               │
├────────────────────────────────────────────────┤
│ Singleton Table                                 │
│  - [SingletonSnapshot serialized...]            │
├────────────────────────────────────────────────┤
│ Spatial Index                                   │
│  - [ChunkId -> Entity list mappings]           │
└────────────────────────────────────────────────┘
```

## Implementation Steps

1. Define `WalEntry` and serialization
2. Implement `WriteAheadLog` with segment files
3. Implement tick index for efficient lookups
4. Define `Snapshot` format and serialization
5. Implement `SnapshotManager`
6. Implement `StorageBackend` trait and `FileBackend`
7. Implement `StorageEngine` with recovery
8. Add time-travel query API
9. Add `MemoryBackend` for testing
10. Optional: Add `LmdbBackend` for production

## Files

```
src/
├── lib.rs          # Re-exports
├── engine.rs       # StorageEngine
├── wal.rs          # WriteAheadLog
├── entry.rs        # WalEntry, TickMutations
├── snapshot.rs     # Snapshot, SnapshotManager
├── backend.rs      # StorageBackend trait
├── file.rs         # FileBackend
├── memory.rs       # MemoryBackend
├── timetravel.rs   # HistoricalWorld
└── format.rs       # Binary serialization
```

## Dependencies

- `rgb-ecs` (World, Entity, Component)
- `crc32fast` (checksums)
- `memmap2` (memory-mapped files, optional)
- `lz4` or `zstd` (compression, optional)

## Configuration

```rust
pub struct StorageConfig {
    /// WAL segment size before rotation
    pub wal_segment_size: u64,

    /// Snapshot policy
    pub snapshot_policy: SnapshotPolicy,

    /// Sync mode
    pub sync_mode: SyncMode,

    /// Compression
    pub compression: Option<Compression>,

    /// Maximum WAL segments to retain
    pub max_wal_segments: usize,
}

pub enum SyncMode {
    /// Sync every tick (safest, slowest)
    EveryTick,

    /// Sync every N ticks
    EveryNTicks(u64),

    /// Sync every N milliseconds
    Periodic(Duration),

    /// Let OS handle (fastest, least safe)
    None,
}
```

## Invariants

1. WAL entries are strictly ordered by tick
2. Snapshots are always at a committed tick boundary
3. Recovery always produces identical state to original
4. Checksums validated on read
5. Concurrent reads allowed, writes are exclusive
