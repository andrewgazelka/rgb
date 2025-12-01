//! Command system with clap integration
//!
//! Handles incoming chat commands and generates Minecraft command tree packets.
//! Uses clap for argument parsing to provide a familiar CLI experience.

use bytes::{BufMut, Bytes, BytesMut};
use mc_protocol::{Decode, Encode};
use rgb_ecs::{Entity, World};
use tracing::{debug, info};

use crate::components::{EntityId, InPlayState, Name, PacketBuffer, Position, Rotation};
use crate::protocol::encode_packet;
use crate::systems::history::{enable_history_by_name, format_history, query_history};

use mc_data::play::clientbound::{Commands, SystemChat};
use mc_data::play::serverbound::ChatCommand;
use mc_protocol::Packet;

/// Serverbound Chat Command packet ID (Play state)
const CHAT_COMMAND_PACKET_ID: i32 = ChatCommand::ID;

/// Clientbound Commands packet ID
const COMMANDS_PACKET_ID: i32 = Commands::ID;

/// Clientbound System Chat packet ID
const SYSTEM_CHAT_PACKET_ID: i32 = SystemChat::ID;

/// Command node flags
const NODE_TYPE_ROOT: u8 = 0;
const NODE_TYPE_LITERAL: u8 = 1;
const NODE_TYPE_ARGUMENT: u8 = 2;
const FLAG_EXECUTABLE: u8 = 0x04;

/// Argument parser IDs from Minecraft registry
mod parser_ids {
    pub const STRING_SINGLE_WORD: i32 = 5; // brigadier:string with single word mode
    pub const INTEGER: i32 = 3; // brigadier:integer
    pub const ENTITY: i32 = 6; // minecraft:entity
}

/// Command definition for building command trees
#[derive(Clone)]
pub struct CommandDef {
    pub name: &'static str,
    pub description: &'static str,
    pub args: Vec<ArgDef>,
}

#[derive(Clone)]
pub struct ArgDef {
    pub name: &'static str,
    pub parser_id: i32,
    pub parser_data: Option<Vec<u8>>,
}

/// All registered commands
pub fn registered_commands() -> Vec<CommandDef> {
    vec![
        CommandDef {
            name: "inspect",
            description: "Inspect an entity's components",
            args: vec![ArgDef {
                name: "entity",
                parser_id: parser_ids::ENTITY,
                // Entity selector flags: single entity, players only = false
                parser_data: Some(vec![0x01]), // SINGLE flag
            }],
        },
        CommandDef {
            name: "history",
            description: "View component history for an entity",
            args: vec![
                ArgDef {
                    name: "entity",
                    parser_id: parser_ids::ENTITY,
                    parser_data: Some(vec![0x01]), // SINGLE flag
                },
                ArgDef {
                    name: "component",
                    parser_id: parser_ids::STRING_SINGLE_WORD,
                    parser_data: Some(vec![0x00]), // SINGLE_WORD mode
                },
            ],
        },
        CommandDef {
            name: "tps",
            description: "Show server TPS",
            args: vec![],
        },
        CommandDef {
            name: "pos",
            description: "Show your position",
            args: vec![],
        },
        CommandDef {
            name: "entities",
            description: "List all entities",
            args: vec![],
        },
        CommandDef {
            name: "track",
            description: "Enable history tracking for entity/component",
            args: vec![
                ArgDef {
                    name: "entity",
                    parser_id: parser_ids::ENTITY,
                    parser_data: Some(vec![0x01]), // SINGLE flag
                },
                ArgDef {
                    name: "component",
                    parser_id: parser_ids::STRING_SINGLE_WORD,
                    parser_data: Some(vec![0x00]), // SINGLE_WORD mode
                },
            ],
        },
    ]
}

/// Write a varint to BytesMut
fn write_varint_bytes(data: &mut BytesMut, mut value: i32) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        data.put_u8(byte);
        if value == 0 {
            break;
        }
    }
}

/// Build the Minecraft command tree packet
pub fn build_commands_packet() -> eyre::Result<Bytes> {
    let commands = registered_commands();
    let mut nodes = Vec::new();

    // Node 0: Root node
    nodes.push(CommandNode {
        flags: NODE_TYPE_ROOT,
        children: Vec::new(),
        redirect: None,
        name: None,
        parser_id: None,
        parser_data: None,
    });

    // Add each command as a literal child of root
    let mut root_children = Vec::new();

    for cmd in &commands {
        let cmd_node_idx = nodes.len() as i32;
        root_children.push(cmd_node_idx);

        if cmd.args.is_empty() {
            // Simple command with no args - executable literal
            nodes.push(CommandNode {
                flags: NODE_TYPE_LITERAL | FLAG_EXECUTABLE,
                children: Vec::new(),
                redirect: None,
                name: Some(cmd.name.to_string()),
                parser_id: None,
                parser_data: None,
            });
        } else {
            // Command with args - literal node followed by argument chain
            let mut arg_indices = Vec::new();

            // Build argument nodes (in reverse so we can chain children)
            for (i, _arg) in cmd.args.iter().enumerate().rev() {
                let arg_idx = nodes.len() as i32 + (cmd.args.len() - 1 - i) as i32 + 1;
                arg_indices.push(arg_idx);
            }
            arg_indices.reverse();

            // Create literal node pointing to first arg
            let first_arg_idx = if arg_indices.is_empty() {
                Vec::new()
            } else {
                vec![nodes.len() as i32 + 1]
            };

            nodes.push(CommandNode {
                flags: NODE_TYPE_LITERAL,
                children: first_arg_idx,
                redirect: None,
                name: Some(cmd.name.to_string()),
                parser_id: None,
                parser_data: None,
            });

            // Add argument nodes
            for (i, arg) in cmd.args.iter().enumerate() {
                let is_last = i == cmd.args.len() - 1;
                let children = if is_last {
                    Vec::new()
                } else {
                    vec![nodes.len() as i32 + 1]
                };

                let flags = NODE_TYPE_ARGUMENT | if is_last { FLAG_EXECUTABLE } else { 0 };

                nodes.push(CommandNode {
                    flags,
                    children,
                    redirect: None,
                    name: Some(arg.name.to_string()),
                    parser_id: Some(arg.parser_id),
                    parser_data: arg.parser_data.clone(),
                });
            }
        }
    }

    // Update root node with children
    nodes[0].children = root_children;

    // Encode packet
    let mut data = BytesMut::new();

    // Node count
    write_varint_bytes(&mut data, nodes.len() as i32);

    // Encode each node
    for node in &nodes {
        data.put_u8(node.flags);

        // Children count and indices
        write_varint_bytes(&mut data, node.children.len() as i32);
        for &child in &node.children {
            write_varint_bytes(&mut data, child);
        }

        // Redirect (optional, not used)
        if node.redirect.is_some() {
            write_varint_bytes(&mut data, node.redirect.unwrap());
        }

        // Name (for literal and argument nodes)
        if let Some(ref name) = node.name {
            let mut name_buf = Vec::new();
            name.clone().encode(&mut name_buf)?;
            data.extend_from_slice(&name_buf);
        }

        // Parser (for argument nodes)
        if let Some(parser_id) = node.parser_id {
            write_varint_bytes(&mut data, parser_id);
            if let Some(ref parser_data) = node.parser_data {
                data.extend_from_slice(parser_data);
            }
        }
    }

    // Root index
    write_varint_bytes(&mut data, 0);

    Ok(data.freeze())
}

struct CommandNode {
    flags: u8,
    children: Vec<i32>,
    redirect: Option<i32>,
    name: Option<String>,
    parser_id: Option<i32>,
    parser_data: Option<Vec<u8>>,
}

/// Send a system chat message to a player
fn send_chat_message(buffer: &mut PacketBuffer, message: &str) {
    let mut data = BytesMut::new();

    // Chat component as NBT (text component)
    let nbt = mc_protocol::nbt! {
        "text" => message,
    };
    data.extend_from_slice(&nbt.to_network_bytes());

    // Overlay = false (not action bar)
    data.put_u8(0);

    buffer.push_outgoing(encode_packet(SYSTEM_CHAT_PACKET_ID, &data));
}

/// Parse a command string and return the command name and arguments
fn parse_command(input: &str) -> Option<(&str, Vec<&str>)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let cmd_name = parts.next()?;
    let args: Vec<&str> = parts.collect();

    Some((cmd_name, args))
}

/// Execute a command and return the response message
fn execute_command(
    cmd: &str,
    args: &[&str],
    executor: Entity,
    world: &World,
) -> Result<String, String> {
    match cmd {
        "tps" => {
            let tps = world
                .get::<crate::components::TpsTracker>(Entity::WORLD)
                .ok_or("TPS tracker not found")?;
            Ok(format!(
                "§aTPS: §f{:.1} §7(5s) §f{:.1} §7(15s) §f{:.1} §7(1m)",
                tps.tps_5s, tps.tps_15s, tps.tps_1m
            ))
        }
        "pos" => {
            let pos = world
                .get::<Position>(executor)
                .ok_or("Position not found")?;
            let rot = world.get::<Rotation>(executor);
            if let Some(rot) = rot {
                Ok(format!(
                    "§aPosition: §f{:.2}, {:.2}, {:.2} §7| Yaw: {:.1} Pitch: {:.1}",
                    pos.x, pos.y, pos.z, rot.yaw, rot.pitch
                ))
            } else {
                Ok(format!(
                    "§aPosition: §f{:.2}, {:.2}, {:.2}",
                    pos.x, pos.y, pos.z
                ))
            }
        }
        "entities" => {
            // Parse optional query DSL from args
            let query_str = if args.is_empty() {
                "*" // Default: all entities
            } else {
                &args.join(" ")
            };

            let query = match query_dsl::parse_query(query_str) {
                Ok(q) => q,
                Err(e) => return Err(format!("§cQuery parse error: {}", e)),
            };

            // Collect matching entities
            let mut results = Vec::new();

            for entity in world.entities_iter() {
                // Skip the WORLD entity
                if entity == rgb_ecs::Entity::WORLD {
                    continue;
                }

                let mut matches = true;
                let mut components_found = Vec::new();

                for term in &query.terms {
                    let component_match = match &term.kind {
                        query_dsl::TermKind::Wildcard => {
                            // Collect all components this entity has
                            if let Some(name) = world.get::<Name>(entity) {
                                components_found.push(format!("Name({})", name.value));
                            }
                            if let Some(pos) = world.get::<Position>(entity) {
                                components_found.push(format!(
                                    "Position({:.1},{:.1},{:.1})",
                                    pos.x, pos.y, pos.z
                                ));
                            }
                            if let Some(rot) = world.get::<Rotation>(entity) {
                                components_found
                                    .push(format!("Rotation({:.0},{:.0})", rot.yaw, rot.pitch));
                            }
                            if let Some(eid) = world.get::<EntityId>(entity) {
                                components_found.push(format!("EntityId({})", eid.value));
                            }
                            if world.has::<InPlayState>(entity) {
                                components_found.push("InPlayState".to_string());
                            }
                            if world.has::<crate::components::Player>(entity) {
                                components_found.push("Player".to_string());
                            }
                            if world.has::<crate::components::Connection>(entity) {
                                components_found.push("Connection".to_string());
                            }
                            if world.has::<crate::components::NeedsSpawnChunks>(entity) {
                                components_found.push("NeedsSpawnChunks".to_string());
                            }
                            true
                        }
                        query_dsl::TermKind::Component(name) => {
                            let has = match name.as_str() {
                                "Position" => world.has::<Position>(entity),
                                "Rotation" => world.has::<Rotation>(entity),
                                "Name" => world.has::<Name>(entity),
                                "EntityId" => world.has::<EntityId>(entity),
                                "InPlayState" => world.has::<InPlayState>(entity),
                                "Player" => world.has::<crate::components::Player>(entity),
                                "Connection" => world.has::<crate::components::Connection>(entity),
                                "NeedsSpawnChunks" => {
                                    world.has::<crate::components::NeedsSpawnChunks>(entity)
                                }
                                "ChunkLoaded" => {
                                    world.has::<crate::components::ChunkLoaded>(entity)
                                }
                                "ChunkPos" => world.has::<crate::components::ChunkPos>(entity),
                                _ => false,
                            };
                            if has && term.operator != query_dsl::Operator::Not {
                                components_found.push(name.clone());
                            }
                            has
                        }
                        query_dsl::TermKind::Pair(_) => {
                            // Pairs not yet supported
                            false
                        }
                    };

                    match term.operator {
                        query_dsl::Operator::And => {
                            if !component_match {
                                matches = false;
                                break;
                            }
                        }
                        query_dsl::Operator::Not => {
                            if component_match {
                                matches = false;
                                break;
                            }
                        }
                        query_dsl::Operator::Optional => {
                            // Optional doesn't affect matching
                        }
                        query_dsl::Operator::Or => {
                            // For simplicity, OR with previous term
                            if component_match {
                                matches = true;
                            }
                        }
                    }
                }

                if matches && !components_found.is_empty() {
                    let eid = world.get::<EntityId>(entity).map(|e| e.value);
                    let name = world.get::<Name>(entity).map(|n| n.value);

                    let label = match (name, eid) {
                        (Some(n), Some(id)) => format!("§f{} §7(id: {})", n, id),
                        (Some(n), None) => format!("§f{}", n),
                        (None, Some(id)) => format!("§7Entity §f{}", id),
                        (None, None) => format!("§7Entity §f{:?}", entity),
                    };

                    results.push(format!(
                        "  §7- {} §8[{}]",
                        label,
                        components_found.join(", ")
                    ));
                }
            }

            if results.is_empty() {
                Ok(format!("§7No entities match query: §f{}", query))
            } else {
                Ok(format!(
                    "§aMatching entities ({}):§r\n{}",
                    results.len(),
                    results.join("\n")
                ))
            }
        }
        "inspect" => {
            if args.is_empty() {
                return Err("Usage: /inspect <entity>".to_string());
            }

            // For now, just inspect self if "@s" or parse entity ID
            let target = if args[0] == "@s" {
                executor
            } else if let Ok(eid) = args[0].parse::<i32>() {
                // Find entity by EntityId
                world
                    .query::<EntityId>()
                    .find(|(_, id)| id.value == eid)
                    .map(|(e, _)| e)
                    .ok_or_else(|| format!("Entity with ID {} not found", eid))?
            } else {
                return Err("Invalid entity selector. Use @s or entity ID".to_string());
            };

            let mut components = Vec::new();

            // Check for known components
            if let Some(name) = world.get::<Name>(target) {
                components.push(format!("§7Name: §f{}", name.value));
            }
            if let Some(pos) = world.get::<Position>(target) {
                components.push(format!(
                    "§7Position: §f{:.2}, {:.2}, {:.2}",
                    pos.x, pos.y, pos.z
                ));
            }
            if let Some(rot) = world.get::<Rotation>(target) {
                components.push(format!(
                    "§7Rotation: §fyaw={:.1} pitch={:.1}",
                    rot.yaw, rot.pitch
                ));
            }
            if let Some(eid) = world.get::<EntityId>(target) {
                components.push(format!("§7EntityId: §f{}", eid.value));
            }
            if world.has::<InPlayState>(target) {
                components.push("§7InPlayState: §atrue".to_string());
            }

            if components.is_empty() {
                Ok("§7No known components found".to_string())
            } else {
                Ok(format!("§aComponents:\n{}", components.join("\n")))
            }
        }
        "history" => {
            if args.len() < 2 {
                return Err(
                    "§cUsage: /history <entity> <component>\n§7Example: /history @s Position"
                        .to_string(),
                );
            }

            // Parse entity selector
            let target = if args[0] == "@s" {
                executor
            } else if let Ok(eid) = args[0].parse::<i32>() {
                world
                    .query::<EntityId>()
                    .find(|(_, id)| id.value == eid)
                    .map(|(e, _)| e)
                    .ok_or_else(|| format!("Entity with ID {} not found", eid))?
            } else {
                return Err("Invalid entity selector. Use @s or entity ID".to_string());
            };

            let component_name = args[1];

            // Normalize component name (capitalize first letter)
            let normalized_name = {
                let mut chars = component_name.chars();
                match chars.next() {
                    None => component_name.to_string(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            };

            // Try to get history first
            if let Some(history) = query_history(world, target, &normalized_name) {
                return Ok(format_history(history, 10));
            }

            // No history tracked - show current value instead
            let current_value = match normalized_name.as_str() {
                "Position" => world
                    .get::<Position>(target)
                    .map(|p| format!("§7Current: §f({:.2}, {:.2}, {:.2})", p.x, p.y, p.z)),
                "Rotation" => world
                    .get::<Rotation>(target)
                    .map(|r| format!("§7Current: §fyaw={:.1}, pitch={:.1}", r.yaw, r.pitch)),
                "Name" => world
                    .get::<Name>(target)
                    .map(|n| format!("§7Current: §f{}", n.value)),
                "EntityId" => world
                    .get::<EntityId>(target)
                    .map(|e| format!("§7Current: §f{}", e.value)),
                _ => None,
            };

            match current_value {
                Some(val) => Ok(format!(
                    "§eNo history recorded yet for {}.\n{}\n§7Use /track @s {} to enable tracking.",
                    normalized_name, val, normalized_name
                )),
                None => Err(format!(
                    "§cComponent '{}' not found on entity or unknown component type.",
                    normalized_name
                )),
            }
        }
        // "track" is handled separately as a deferred command
        "track" => Err("__DEFERRED_TRACK__".to_string()),
        _ => Err(format!("§cUnknown command: /{}", cmd)),
    }
}

/// Deferred command that needs mutable world access
pub enum DeferredCommand {
    EnableTracking {
        entity: Entity,
        component_name: String,
        response_to: Entity,
    },
}

/// Parse the /track command and return a deferred command
fn parse_track_command(
    args: &[&str],
    executor: Entity,
    world: &World,
) -> Result<DeferredCommand, String> {
    if args.len() < 2 {
        return Err(
            "§cUsage: /track <entity> <component>\n§7Example: /track @s Position".to_string(),
        );
    }

    // Parse entity selector
    let target = if args[0] == "@s" {
        executor
    } else if let Ok(eid) = args[0].parse::<i32>() {
        world
            .query::<EntityId>()
            .find(|(_, id)| id.value == eid)
            .map(|(e, _)| e)
            .ok_or_else(|| format!("Entity with ID {} not found", eid))?
    } else {
        return Err("Invalid entity selector. Use @s or entity ID".to_string());
    };

    let component_name = args[1];

    // Normalize component name (capitalize first letter)
    let normalized_name = {
        let mut chars = component_name.chars();
        match chars.next() {
            None => component_name.to_string(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    };

    // Check if already tracked
    if query_history(world, target, &normalized_name).is_some() {
        return Err(format!(
            "§eHistory tracking already enabled for {} on this entity.",
            normalized_name
        ));
    }

    Ok(DeferredCommand::EnableTracking {
        entity: target,
        component_name: normalized_name,
        response_to: executor,
    })
}

/// Execute a deferred command that needs mutable world access
pub fn execute_deferred_command(world: &mut World, cmd: DeferredCommand) {
    match cmd {
        DeferredCommand::EnableTracking {
            entity,
            component_name,
            response_to,
        } => {
            let response = match enable_history_by_name(world, entity, &component_name) {
                Ok(()) => format!(
                    "§aEnabled history tracking for {} on entity.",
                    component_name
                ),
                Err(e) => format!("§cFailed to enable tracking: {}", e),
            };

            if let Some(mut buffer) = world.get::<PacketBuffer>(response_to) {
                send_chat_message(&mut buffer, &response);
                world.update(response_to, buffer);
            }
        }
    }
}

/// System: Handle incoming chat commands
pub fn system_handle_commands(world: &mut World) {
    let play_entities: Vec<_> = world
        .query::<InPlayState>()
        .map(|(entity, _)| entity)
        .collect();

    // Collect deferred commands to execute after the main loop
    let mut deferred_commands = Vec::new();

    for executor in play_entities {
        let Some(mut buffer) = world.get::<PacketBuffer>(executor) else {
            continue;
        };

        let mut commands_to_execute = Vec::new();
        let mut remaining = Vec::new();

        while let Some((packet_id, data)) = buffer.pop_incoming() {
            if packet_id == CHAT_COMMAND_PACKET_ID {
                // Parse command string
                let mut cursor = std::io::Cursor::new(&data[..]);
                if let Ok(command_str) = String::decode(&mut cursor) {
                    commands_to_execute.push(command_str);
                }
            } else {
                remaining.push((packet_id, data));
            }
        }

        // Put remaining packets back
        for (id, data) in remaining {
            buffer.push_incoming(id, data);
        }

        // Execute commands
        for command_str in commands_to_execute {
            let executor_name = world
                .get::<Name>(executor)
                .map(|n| n.value)
                .unwrap_or_else(|| "Unknown".to_string());

            info!("{} executed command: /{}", executor_name, command_str);

            if let Some((cmd, args)) = parse_command(&command_str) {
                // Handle deferred commands (those needing &mut World)
                if cmd == "track" {
                    match parse_track_command(&args, executor, world) {
                        Ok(deferred) => deferred_commands.push(deferred),
                        Err(err) => send_chat_message(&mut buffer, &err),
                    }
                    continue;
                }

                let response = match execute_command(cmd, &args, executor, world) {
                    Ok(msg) => msg,
                    Err(err) => err,
                };
                send_chat_message(&mut buffer, &response);
            }
        }

        world.update(executor, buffer);
    }

    // Execute deferred commands that need &mut World
    for cmd in deferred_commands {
        execute_deferred_command(world, cmd);
    }
}

/// System: Send command tree to new players
///
/// Should be called when a player enters play state
pub fn send_commands_to_player(buffer: &mut PacketBuffer) {
    match build_commands_packet() {
        Ok(data) => {
            buffer.push_outgoing(encode_packet(COMMANDS_PACKET_ID, &data));
            debug!("Sent command tree to player");
        }
        Err(e) => {
            tracing::error!("Failed to build commands packet: {}", e);
        }
    }
}
