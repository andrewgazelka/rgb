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

## Hot-Reloadable Plugins

The server supports hot-reloadable Flecs modules via Rust dylibs.

### Rust Version Pinning (CRITICAL)

**All plugins MUST be compiled with the exact same Rust toolchain version.** Rust does not guarantee ABI stability across versions. The project uses:

- **Rust version**: Pinned via `rust-toolchain.toml` or Nix flake
- **Edition**: 2024

When building plugins:
1. Always use the same `rustc` version as the main server
2. Never mix plugins compiled with different Rust versions
3. Use `nix build` to ensure consistent toolchain

### Plugin Structure

Each plugin is a `cdylib` crate that exports:

```rust
#[unsafe(no_mangle)]
pub extern "Rust" fn plugin_load(world: &World) { ... }

#[unsafe(no_mangle)]
pub extern "Rust" fn plugin_unload(world: &World) { ... }

#[unsafe(no_mangle)]
pub extern "Rust" fn plugin_name() -> &'static str { ... }
```

### Plugin Directory

Place compiled `.dylib` (macOS), `.so` (Linux), or `.dll` (Windows) files in the `plugins/` directory. The server will:

1. Load all plugins on startup
2. Watch for file changes
3. Hot-reload modified plugins automatically
