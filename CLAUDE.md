Things that have no self refernce should be free funciton not associated function like Self::get_cell

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

## Hot-Reloadable Plugins (WIP)

**STATUS: NOT YET WORKING** - Plugin infrastructure exists but component sharing across dylibs fails.

### The Problem

Flecs hot-reloading with Rust dylibs has a fundamental issue: each dylib gets its own copy of the `flecs_ecs` Rust bindings, which have their own component ID maps. Even when sharing the flecs C library symbols, the Rust-side component registration is duplicated, causing "mismatching size for field" errors.

### Solution: Shared `flecs_ecs` dylib

Build `flecs_ecs` as a Rust dylib (`crate-type = ["dylib"]`) so both host and plugins link to a single shared `libflecs_ecs.dylib`.

**Rust ABI stability**: We accept unstable Rust ABI because we pin the compiler version via Nix flake. All plugins MUST be compiled with the exact same `rustc` version as the host.

### Alternative Solutions

1. **Use a scripting language** - Lua, WASM, or Rhai for plugin logic
2. **Use flecs C API directly** - Bypass the Rust bindings entirely in plugins

### Current Infrastructure

The plugin loading infrastructure is in place:
- `plugin-loader` crate: Dynamic library loading with file watching
- `mc-server-runner` binary: Loads plugins from `plugins/` directory
- `plugin-time` example: Template for plugin structure

### Plugin Structure (C ABI)

Plugins use C ABI to avoid Rust ABI issues:

```rust
use flecs_ecs::core::WorldRef;
use flecs_ecs::sys;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_load(world_ptr: *mut sys::ecs_world_t) {
    let world = unsafe { WorldRef::from_ptr(world_ptr) };
    // ... register module
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_unload(world_ptr: *mut sys::ecs_world_t) {
    // ... cleanup
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_name() -> *const std::ffi::c_char {
    c"my-plugin".as_ptr()
}
```

### Linking Plugins (Development)

```bash
./scripts/link-plugins.sh debug  # Symlink debug builds to plugins/
./scripts/link-plugins.sh release  # Symlink release builds
```
