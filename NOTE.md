# RGB Minecraft Server - Architecture Notes

## ECS Migration (Flecs → RGB ECS)

The server was migrated from Flecs ECS to a custom RGB ECS implementation.

### Key Differences

**Global State**: Stored on `Entity::WORLD` instead of singletons
```rust
world.insert(Entity::WORLD, WorldTime::default());
let time = world.get::<WorldTime>(Entity::WORLD)?;
```

**Owned Value Pattern**: Components are cloned on read, written back on update
```rust
let mut buffer = world.get::<PacketBuffer>(entity)?;
buffer.push_outgoing(packet);
world.update(entity, buffer);
```

**Auto-Registration**: Components register automatically on first use - no explicit registration needed

**Manual Systems**: Systems are plain functions called each tick (no scheduler)
```rust
pub fn tick(world: &mut World, delta_time: f32) {
    network::system_network_ingress(world);
    handshake::system_handle_handshake(world);
    // ...
}
```

### Crate Structure

```
crates/
├── rgb-ecs/        # Core ECS (World, Entity, Component, Archetype)
├── rgb-spatial/    # Spatial partitioning with RGB coloring
├── rgb-query/      # Query scopes for neighborhood access
├── rgb-storage/    # RocksDB-backed persistence
├── rgb-tick/       # Tick-based execution framework
└── mc-server-runner/
    ├── src/
    │   ├── main.rs         # Entry point, game loop
    │   ├── components.rs   # All ECS components
    │   ├── network.rs      # Async TCP server
    │   ├── protocol.rs     # Packet encoding helpers
    │   ├── registry.rs     # MC registry data
    │   ├── world_gen.rs    # Dune terrain generation
    │   └── systems/        # Game logic
    │       ├── mod.rs      # System orchestration
    │       ├── network.rs  # Ingress/egress
    │       ├── handshake.rs
    │       ├── login.rs
    │       ├── config.rs
    │       ├── play.rs
    │       └── time.rs
```

### Running

```bash
cargo run -p mc-server-runner
```

Environment variables:
- `TARGET_FPS` - Ticks per second (default: 20)
- `MC_PORT` - Server port (default: 25565)
