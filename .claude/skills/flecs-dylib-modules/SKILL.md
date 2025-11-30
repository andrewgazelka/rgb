---
name: flecs-dylib-plugins
description: Hot-reloadable Flecs modules as Rust dylibs. Covers plugin architecture, singleton setup, and inter-plugin dependencies.
---

# Flecs Dylib Plugins

Hot-reloadable Flecs modules as Rust dylibs.

## When to Use

Use this skill when you need to:
- Create a new hot-reloadable plugin
- Understand how plugins depend on each other
- Debug singleton/component registration issues

## Architecture

Each plugin is a separate dylib crate that:
1. Defines its own components/singletons
2. Exports a Flecs `Module`
3. Other plugins can depend on it via Cargo and use its types

## Key Insight: No Central Types Crate Needed

Plugins export their types directly. Other plugins depend on them:

```rust
// crates/plugin-time/src/lib.rs (dylib)
#[derive(Component)]
pub struct WorldTime {
    pub world_age: i64,
    pub time_of_day: i64,
}

#[derive(Component)]
pub struct TimeModule;

impl Module for TimeModule {
    fn module(world: &World) {
        world.module::<TimeModule>("time");
        world.component::<WorldTime>().add_trait::<flecs::Singleton>();
        world.set(WorldTime::default());
        // systems...
    }
}
```

```rust
// crates/plugin-play/src/lib.rs (dylib)
use plugin_time::{TimeModule, WorldTime};  // Cargo dep on plugin-time

#[derive(Component)]
pub struct PlayModule;

impl Module for PlayModule {
    fn module(world: &World) {
        world.module::<PlayModule>("play");
        world.import::<TimeModule>();  // Ensures singleton exists

        world.system::<&WorldTime>()  // Use the type directly
            .each(|time| { /* ... */ });
    }
}
```

## Cargo.toml Setup

```toml
# crates/plugin-time/Cargo.toml
[lib]
crate-type = ["dylib"]

[dependencies]
flecs_ecs.workspace = true
```

```toml
# crates/plugin-play/Cargo.toml
[lib]
crate-type = ["dylib"]

[dependencies]
flecs_ecs.workspace = true
plugin-time = { path = "../plugin-time" }  # For types only
```

## Plugin Interface (Rust ABI)

Each plugin exports these functions:

```rust
#[unsafe(no_mangle)]
pub fn plugin_load(world: &World) {
    world.import::<MyModule>();
}

#[unsafe(no_mangle)]
pub fn plugin_unload(world: &World) {
    if let Some(e) = world.try_lookup("::my_module") {
        e.destruct();
    }
}

#[unsafe(no_mangle)]
pub fn plugin_name() -> &'static str { "my_module" }

#[unsafe(no_mangle)]
pub fn plugin_version() -> u32 { 1 }
```

## Critical: Singleton Setup

Modules that define singletons MUST set them up:

```rust
// WRONG - just registers component
world.component::<WorldTime>();

// RIGHT - registers as singleton AND sets initial value
world.component::<WorldTime>().add_trait::<flecs::Singleton>();
world.set(WorldTime::default());
```

## Critical: Import Order via world.import()

`world.import::<Module>()` is idempotent. Call it to declare dependencies:

```rust
impl Module for PlayModule {
    fn module(world: &World) {
        world.import::<TimeModule>();   // Ensures TimeModule loaded first
        world.import::<ChunkModule>();  // Ensures ChunkModule loaded first
        // Now safe to query WorldTime, ChunkIndex, etc.
    }
}
```

Load order of dylibs doesn't matter - each plugin's `import` calls handle dependencies.

## Shared Libraries Required

Both host and all plugins must link to the SAME:
- `libflecs.dylib` (C library)
- `libflecs_ecs.dylib` (Rust wrapper)

In flecs_ecs Cargo.toml:
```toml
[lib]
crate-type = ["dylib"]
```

## Symlink for Development

During dev, symlink built dylibs to plugins/:
```bash
ln -s target/debug/libplugin_time.dylib plugins/
```

Rebuild updates the dylib, file watcher triggers hot-reload.
