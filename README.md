<p align="center">
  <img src=".github/assets/header.svg" alt="rgb" width="100%"/>
</p>

<p align="center">
  <code>nix run github:andrewgazelka/rgb</code>
</p>

A high-performance Minecraft server with hot-reloadable plugins and flat, readable code.

## Features

- **Hot-Reloadable Plugins**: Modify Rust dylib plugins or Skript handlers without restarting the server
- **Data-Parallel Architecture**: Operations grouped by type (R, G, B), executed in parallel across all entities—no per-entity sequential loops
- **Flat, Explicit Code**: No abstraction layers hiding behavior. Every line does exactly what it says.
- **Latest Snapshots**: Targets Minecraft snapshot builds to stay ahead

## How It Works

The animation shows the parallelism model: all same-colored blocks pulse together. This is how the server processes entities—group by operation type, execute each group in parallel, then move to the next. Pure data-parallel execution.

## Plugin Systems

- **Skript Compatibility**: Hot-reloadable event handlers using the [Skript](https://github.com/SkriptLang/Skript) scripting language. The `skript-lang` crate provides parsing and AST for Skript files.
- **Rust Dylib Plugins**: Native Rust plugins that can be hot-reloaded at runtime.

## Status

Early development. Targeting Minecraft snapshot builds.

```
cargo test        # run tests
./ci.sh           # fmt, clippy, tests
```

---

<details>
<summary>About this project</summary>

This codebase is 100% AI-authored (Claude Opus 4.5). Both client and server are in Rust so the AI can spin up tests and iterate autonomously—just `cargo test`, no manual Java clients. Minecraft removed obfuscation in snapshot 25w46a, enabling direct source matching.

</details>
