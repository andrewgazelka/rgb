# Plan: `rgb-commands` Crate - Clap-based Minecraft Commands

## Overview

Create a new `rgb-commands` crate that allows defining Minecraft commands using Clap's `#[derive(Parser)]` macro. Commands register to a global command tree (stored as Flecs entities with relations), and the tree is serialized to Minecraft's `ClientboundCommandsPacket` format for client autocomplete.

## Key Design Decisions

1. **Clap-first**: Commands are plain Rust structs with `#[derive(clap::Parser)]`
2. **Flecs relations**: Command tree nodes are entities; executors attached via relations
3. **Execution**: `fn execute(&self, ctx: CommandCtx)` where `ctx` provides world access
4. **MC argument types**: Deferred to future work (TODO documented)
5. **Timing**: Command tree sent during configuration phase (vanilla behavior)

## Crate Structure

```
crates/rgb-commands/
  Cargo.toml
  src/
    lib.rs              # Public API, CommandsModule, register_command!
    tree.rs             # Command tree types (nodes as Flecs entities)
    packet.rs           # ClientboundCommandsPacket encoding
    context.rs          # CommandCtx for execution
    execute.rs          # Command trait and executor component
```

## Core Types

### Command Trait

```rust
/// Trait for commands that can be registered and executed
pub trait Command: clap::Parser + Send + Sync + 'static {
    /// Execute the command
    fn execute(&self, ctx: &CommandCtx) -> CommandResult;
}

pub struct CommandCtx<'w> {
    pub executor: EntityView<'w>,
    pub world: WorldRef<'w>,
}

pub type CommandResult = Result<(), CommandError>;
```

### Tree as Flecs Entities

```rust
// Node types (tags)
#[derive(Component, Default)]
pub struct RootNode;

#[derive(Component)]
pub struct LiteralNode {
    pub name: String,
}

#[derive(Component)]
pub struct ArgumentNode {
    pub name: String,
    pub arg_type: ArgumentType, // For now: just String variants
}

// Relation: parent -> child
#[derive(Component)]
pub struct ChildOf;  // Use Flecs built-in or custom

// Executor component (attached to executable nodes)
#[derive(Component)]
pub struct Executor {
    pub handler: fn(&[u8], &CommandCtx) -> CommandResult,
}
```

### Registration API

```rust
/// Register a Clap command to the global tree
pub fn register_command<C: Command>(world: &World) {
    let clap_cmd = C::command();
    let tree_root = get_or_create_root(world);

    // Convert Clap command to tree nodes
    let node = create_literal_node(world, clap_cmd.get_name());
    world.entity_from_id(node).add_pair::<ChildOf>(tree_root);

    // Add arguments as child nodes
    for arg in clap_cmd.get_positionals() {
        let arg_node = create_argument_node(world, arg);
        world.entity_from_id(arg_node).add_pair::<ChildOf>(node);
    }

    // Attach executor to leaf node
    attach_executor::<C>(world, node);
}
```

## Packet Encoding

The `ClientboundCommandsPacket` format (packet ID 16 in Configuration state):

1. Flatten tree to array via BFS traversal
2. For each node encode:
   - `flags: u8` (type bits 0-1, executable bit 2, redirect bit 3, suggestions bit 4)
   - `children: VarInt[]` (indices)
   - `redirect: VarInt` (optional)
   - Type-specific: Literal=name, Argument=name+type_id+type_data
3. `root_index: VarInt`

```rust
pub fn encode_commands_packet(world: &World) -> Vec<u8> {
    let root = get_root_node(world);
    let nodes = collect_nodes_bfs(world, root);

    let mut buf = Vec::new();
    write_varint(&mut buf, nodes.len() as i32);

    for (idx, node) in nodes.iter().enumerate() {
        encode_node(&mut buf, node, &nodes);
    }

    write_varint(&mut buf, 0); // root is always index 0
    buf
}
```

## Flecs Module Integration

```rust
#[derive(Component)]
pub struct CommandsModule;

impl Module for CommandsModule {
    fn module(world: &World) {
        world.module::<CommandsModule>("commands");

        // Register components
        world.component::<RootNode>();
        world.component::<LiteralNode>();
        world.component::<ArgumentNode>();
        world.component::<Executor>();

        // Create root node singleton
        world.entity_named("command_root").add::<RootNode>();

        // Register packet handler for chat commands (Play state)
        world.register_handler(
            "ChatCommand",
            ConnectionState::Play,
            6, // minecraft:chat_command serverbound
            0,
            handle_chat_command,
        );
    }
}

fn handle_chat_command(entity: EntityView<'_>, data: &[u8]) {
    // Parse command string
    // Walk tree to find matching node
    // Parse args with Clap
    // Call executor
}
```

## Configuration Phase Integration

Modify `config.rs` to send command tree after registry data:

```rust
// In send_registry_data(), after all registries:
send_commands_packet(buffer, world);

fn send_commands_packet(buffer: &mut PacketBuffer, world: WorldRef<'_>) {
    let data = encode_commands_packet(world);
    let packet = encode_packet(16, &data); // Commands packet ID in config
    buffer.push_outgoing(packet);
}
```

## Example Usage

```rust
use clap::Parser;
use rgb_commands::{Command, CommandCtx, CommandResult};

#[derive(Parser)]
#[command(name = "hello")]
pub struct HelloCommand {
    /// Name to greet
    name: String,
}

impl Command for HelloCommand {
    fn execute(&self, ctx: &CommandCtx) -> CommandResult {
        // Send chat message to executor
        send_chat(ctx.executor, &format!("Hello, {}!", self.name));
        Ok(())
    }
}

// In module setup:
rgb_commands::register_command::<HelloCommand>(world);
```

## Implementation Steps

### Phase 1: Core Crate Setup
- [ ] Create `crates/rgb-commands/Cargo.toml` with deps (clap, flecs_ecs, mc-protocol)
- [ ] Define `Command` trait in `lib.rs`
- [ ] Define `CommandCtx` and `CommandResult`

### Phase 2: Tree Structure
- [ ] Create node components (`RootNode`, `LiteralNode`, `ArgumentNode`)
- [ ] Implement `register_command<C>()` that converts Clap metadata to tree nodes
- [ ] Use Flecs entity relations for parent-child structure

### Phase 3: Packet Encoding
- [ ] Implement BFS tree traversal to flatten nodes
- [ ] Implement `encode_commands_packet()` following Minecraft protocol
- [ ] For now: all arguments encoded as `brigadier:string` (word mode)

### Phase 4: Execution
- [ ] Implement `handle_chat_command` packet handler
- [ ] Parse incoming command, match against tree
- [ ] Call Clap to parse arguments, invoke executor

### Phase 5: Integration
- [ ] Add `CommandsModule` Flecs module
- [ ] Modify `config.rs` to send command packet after registries
- [ ] Add example command (`/hello`)

### Phase 6: Testing
- [ ] Test with real Minecraft client
- [ ] Verify autocomplete works
- [ ] Verify command execution works

## Critical Files to Modify

1. **Create**: `crates/rgb-commands/Cargo.toml`
2. **Create**: `crates/rgb-commands/src/lib.rs` (+ submodules)
3. **Modify**: `crates/mc-server-lib/src/modules/config.rs` - send command tree
4. **Modify**: `crates/mc-server-lib/src/lib.rs` - import CommandsModule
5. **Modify**: `Cargo.toml` (workspace) - add rgb-commands member

## Future Work (TODO)

- [ ] Minecraft-specific argument types (Entity, Vec3, BlockPos, etc.)
- [ ] Permission system (op levels)
- [ ] Suggestion providers for autocomplete
- [ ] Subcommand support
- [ ] Command unregistration for hot-reload

## Reference: Minecraft Protocol Details

### Relevant Packets
- `minecraft:commands` (Clientbound, Config state, ID 16) - sends command tree
- `minecraft:chat_command` (Serverbound, Play state, ID 6) - receives command
- `minecraft:command_suggestion` (Serverbound, Play state, ID 14) - autocomplete request
- `minecraft:command_suggestions` (Clientbound, Play state, ID 15) - autocomplete response

### Decompiled Source References
- `/tmp/mc-decompile-1.21.11-pre3/decompiled/net/minecraft/network/protocol/game/ClientboundCommandsPacket.java`
- `/tmp/mc-decompile-1.21.11-pre3/decompiled/net/minecraft/commands/Commands.java`
- `/tmp/mc-decompile-1.21.11-pre3/decompiled/net/minecraft/network/protocol/game/ServerboundChatCommandPacket.java`
