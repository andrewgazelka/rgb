Things that have no self refernce should be free funciton not associated function like Self::get_cell

All projects are in /Users/andrewgazelka/Projects/{name}; look there for projects ; i.e., /Users/andrewgazelka/Projects/flecs

I am PINNING THE RUST toolchain so i would imagine is not an issue no ABI issues

NEVER use cdylib ; use rust dylib (assume compiler versions match)

Prefer almost always using inline path like `core::mem::slice` instead of `use core::mem::slice` `slice::...`

Run `./ci.sh` for CI checks (fmt, clippy, tests)

Use `cargo nextest run` for tests

Avoid anonymous tuples with >2 elements. Use named structs instead for clarity.

## Dependencies

All dependencies must be defined at the workspace level in the root `Cargo.toml`, then referenced with `.workspace = true` in individual crates:

```toml
# Root Cargo.toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }

# Crate Cargo.toml
[dependencies]
tokio.workspace = true
```

## Project: Minecraft Server in Rust

This is a Minecraft server implementation targeting version 1.21.11-pre3.

### Decompiled Minecraft Source Code

**CRITICAL**: The decompiled Minecraft source is the **SINGLE SOURCE OF TRUTH**. Always consult it first before web searches or wikis. The wiki can be outdated or wrong - the decompiled code never lies.

Decompiled source location: `/tmp/mc-decompile-1.21.11-pre3/decompiled/net/minecraft/`

To decompile 1.21.11-pre3:
```bash
# Download client jar (get hash from version manifest)
curl -o client.jar 'https://piston-data.mojang.com/v1/objects/<hash>/client.jar'
# Decompile with CFR
java -jar cfr.jar client.jar --outputdir /tmp/mc-decompile-1.21.11-pre3/decompiled
```

**Key decompiled locations**:
- Packets: `network/protocol/game/`
- Codecs: `network/codec/ByteBufCodecs.java`
- Chunk sections: `world/level/chunk/`
- Paletted containers: `world/level/chunk/PalettedContainer.java`
- Biomes/Sky color: `data/worldgen/biome/OverworldBiomes.java`
- Dimension types: `data/worldgen/DimensionTypes.java`
- Light data: `network/protocol/game/ClientboundLightUpdatePacketData.java`
- Environment attributes (sky_color, fog_color): `world/attribute/EnvironmentAttributes.java`
- Attribute map serialization: `world/attribute/EnvironmentAttributeMap.java`
- Sky rendering: `client/renderer/SkyRenderer.java`
- Blocks registry: `world/level/block/Blocks.java`

### Generated Data

Use `nix run .#mc-data-gen` to generate Mojang's data reports (blocks, items, packets, etc).

Data files in `crates/mc-data/data/`:
- `blocks.json` - All block states with IDs (from Mojang data generator)
- `packets-ids.json` - Packet IDs
- `protocol.json` - Protocol version info

### Block Registry

Block state IDs are auto-generated from `blocks.json`. Use the registry instead of hardcoding IDs:

```rust
use mc_data::{BlockState, blocks};

// Use block constants (default states)
let air = blocks::AIR;           // BlockState(0)
let bedrock = blocks::BEDROCK;   // BlockState(85)
let dirt = blocks::DIRT;         // BlockState(10)
let grass = blocks::GRASS_BLOCK; // BlockState(9)

// Get raw ID for protocol
let id: u16 = bedrock.id();

// Lookup by name
let stone = BlockState::by_name("minecraft:stone");
```

**NEVER hardcode block state IDs** - they change between versions. Always use the registry.

## Hot-Reloadable Modules (WIP)

**STATUS: NOT YET WORKING** - Module infrastructure exists but component sharing across dylibs fails.

### The Problem

Flecs hot-reloading with Rust dylibs has a fundamental issue: each dylib gets its own copy of the `flecs_ecs` Rust bindings, which have their own component ID maps. Even when sharing the flecs C library symbols, the Rust-side component registration is duplicated, causing "mismatching size for field" errors.

### Solution: Shared `flecs_ecs` dylib

Build `flecs_ecs` as a Rust dylib (`crate-type = ["dylib"]`) so both host and modules link to a single shared `libflecs_ecs.dylib`.

**Rust ABI stability**: We accept unstable Rust ABI because we pin the compiler version via Nix flake. All modules MUST be compiled with the exact same `rustc` version as the host.

### Alternative Solutions

1. **Use a scripting language** - Lua, WASM, or Rhai for module logic
2. **Use flecs C API directly** - Bypass the Rust bindings entirely in modules

### Current Infrastructure

The module loading infrastructure is in place:
- `module-loader` crate: Dynamic library loading with file watching
- `mc-server-runner` binary: Loads modules from `modules/` directory
- `module-time` example: Template for module structure

### Module Structure (C ABI)

Modules use C ABI to avoid Rust ABI issues:

```rust
use flecs_ecs::core::WorldRef;
use flecs_ecs::sys;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn module_load(world_ptr: *mut sys::ecs_world_t) {
    let world = unsafe { WorldRef::from_ptr(world_ptr) };
    // ... register module
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn module_unload(world_ptr: *mut sys::ecs_world_t) {
    // ... cleanup
}

#[unsafe(no_mangle)]
pub extern "C" fn module_name() -> *const std::ffi::c_char {
    c"my-module".as_ptr()
}
```

### Linking Modules (Development)

```bash
./scripts/link-modules.sh debug  # Symlink debug builds to modules/
./scripts/link-modules.sh release  # Symlink release builds
```

## ECS Design Principles

### NO LOCKS - CRITICAL RULE

**NEVER use `Mutex`, `RwLock`, or any synchronization primitives in this codebase.**

The ECS handles all synchronization. If you think you need a lock, **STOP and ask the user first** - there's always a better ECS pattern:
- Need shared mutable state? → Use a Resource
- Need per-entity mutable data? → Use components with owned-value access
- Need cross-system communication? → Use events or relations

If Claude thinks a lock is needed, it MUST alert the user and stop before proceeding.

### Owned Values, Not References

RGB uses **owned values** for component access (SpacetimeDB-style), not references:

```rust
// Get returns owned value (cloned from storage)
let pos = world.get::<Position>(entity)?;

// Modify and write back
let new_pos = Position { x: pos.x + 1.0, ..pos };
world.update(entity, new_pos);
```

**Why**: Enables hot-reloadable modules, avoids lifetime complexity, works across FFI boundaries.

### Component Types: POD vs Opaque

Components must use `#[derive(Component)]`. There are two kinds:

#### POD Components (Default)
Simple, flat data. No heap allocations, no collections. Validated at compile time.

```rust
#[derive(Component, Clone)]
struct Position { x: f64, y: f64, z: f64 }

#[derive(Component, Clone)]
struct Health { current: u32, max: u32 }
```

**Forbidden types** (compile error with helpful message):
- `Vec`, `VecDeque`, `HashMap`, `HashSet` → Use relations instead
- `String` → Use `[u8; N]` or `world.set_name()`
- `Box`, `Rc`, `Arc` → Use entity references
- `Sender`, `Receiver` → Use resources or events

**Allowed types** (O(1) clone, safe for components):
- `bytes::Bytes` - Arc-based, cheap clone

#### Opaque Components
Runtime handles that can't be POD. Use sparingly.

```rust
#[derive(Component, Clone)]
#[component(opaque)]
struct NetworkIngress {
    rx: Receiver<Packet>,
}
```

**When to use opaque:**
- Global singletons (network channels, file handles)
- External system handles
- Caches that reference external state

**Prefer relations over opaque** for per-entity data.

### Relations Over Collections

Instead of storing collections in components, use entity relations:

```rust
// BAD: Collection in component (expensive clone, can't query)
#[derive(Component)]
struct Inventory { items: Vec<Item> }  // Compile error!

// GOOD: Each item is an entity with a relation
#[derive(Component, Clone)]
struct Item { id: u32, count: u8, slot: u8 }

// Spawn items with relation to player
world.spawn((Item { id: 1, count: 64, slot: 0 }, Pair::<ChildOf>(player)));
world.spawn((Item { id: 2, count: 32, slot: 1 }, Pair::<ChildOf>(player)));

// Query all items for a player
let query = world.query()
    .with::<Item>()
    .pair::<ChildOf>(player)
    .build();
```

**Benefits:**
- No cloning collections on every access
- Items are queryable and indexable
- Natural ordering with `order_by`
- Parallel-safe (different parents = different archetypes)

### Runtime Query Builder (Flecs-Rust Style)

Queries should use a **runtime builder pattern**, NOT heavy type-level generics:

```rust
// GOOD: Runtime builder (FFI-friendly, simple, flexible)
let query = world.query()
    .with::<Position>()
    .with::<Velocity>()
    .optional::<Health>()
    .without::<Dead>()
    .build();

for result in query.iter(&world) {
    let entity = result.entity();
    let pos: Position = result.get::<Position>();
    let vel: Velocity = result.get::<Velocity>();
    let health: Option<Health> = result.get_optional::<Health>();
}

// BAD: Heavy type-level generics (hard to FFI, complex error messages)
let query = world.query::<(Fetch<Position>, Fetch<Velocity>, Option<Fetch<Health>>, Without<Dead>)>();
```

**Why**:
1. **FFI-friendly** - Can expose to Lua, WASM, other languages
2. **Simpler errors** - No 500-line generic error messages
3. **Dynamic composition** - Build queries from runtime data
4. **Matches Flecs** - Consistent with Flecs-Rust patterns we're familiar with
