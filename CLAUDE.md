Things that have no self refernce should be free funciton not associated function like Self::get_cell

Run `./ci.sh` for CI checks (fmt, clippy, tests)

Use `cargo nextest run` for tests

Avoid anonymous tuples with >2 elements. Use named structs instead for clarity.

## Project: Minecraft Server in Rust

This is a Minecraft server implementation targeting version 1.21.11-pre3.

### Decompiled Minecraft Source Code

**IMPORTANT**: Always look within the decompiled Minecraft code for protocol details instead of using web search. The decompiled code is the authoritative source for packet formats.

Minecraft removed obfuscation starting with snapshot **25w46a** (first snapshot after Mounts of Mayhem launch). For earlier versions like 1.21.11-pre3, the code is still obfuscated.

To get readable source code, decompile a **post-obfuscation-removal snapshot** (25w46a or later):
- Download: `https://piston-data.mojang.com/v1/objects/{hash}/client.jar`
- Decompile with CFR: `java -jar cfr.jar client.jar --outputdir /tmp/mc-decompile-new/decompiled`
- Use `/tmp/mc-decompile-new/decompiled/net/minecraft/` for readable class names

The packet format is the same between versions - only the class/method names differ.

Key decompiled locations:
- Packets: `/tmp/mc-decompile-new/decompiled/net/minecraft/network/protocol/game/`
- Codecs: `/tmp/mc-decompile-new/decompiled/net/minecraft/network/codec/ByteBufCodecs.java`
- Chunk sections: `/tmp/mc-decompile-new/decompiled/net/minecraft/world/level/chunk/`
- Paletted containers: `/tmp/mc-decompile-new/decompiled/net/minecraft/world/level/chunk/PalettedContainer.java`

