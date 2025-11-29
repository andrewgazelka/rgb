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

