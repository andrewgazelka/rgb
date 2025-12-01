//! Command system with clap integration
//!
//! Handles incoming chat commands and generates Minecraft command tree packets.
//! Uses clap for argument parsing to provide a familiar CLI experience.

use std::collections::HashMap;

use bytes::{BufMut, Bytes, BytesMut};
use mc_protocol::{Encode, write_varint};
use rgb_ecs::{Entity, World};
use tracing::{debug, info};

use crate::components::{
    ConnectionIndex, EntityId, InPlayState, Name, PacketBuffer, Position, Rotation,
};
use crate::protocol::encode_packet;

/// Serverbound Chat Command packet ID (Play state)
const CHAT_COMMAND_PACKET_ID: i32 = 0x06;

/// Clientbound Commands packet ID
const COMMANDS_PACKET_ID: i32 = 0x10;

/// Clientbound System Chat packet ID
const SYSTEM_CHAT_PACKET_ID: i32 = 0x73;

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
                parser_data: Some({
                    let mut data = Vec::new();
                    // Entity selector flags: single entity, players only = false
                    data.push(0x01); // SINGLE flag
                    data
                }),
            }],
        },
        CommandDef {
            name: "history",
            description: "View component history for an entity",
            args: vec![
                ArgDef {
                    name: "entity",
                    parser_id: parser_ids::ENTITY,
                    parser_data: Some({
                        let mut data = Vec::new();
                        data.push(0x01); // SINGLE flag
                        data
                    }),
                },
                ArgDef {
                    name: "component",
                    parser_id: parser_ids::STRING_SINGLE_WORD,
                    parser_data: Some({
                        let mut data = Vec::new();
                        write_varint(&mut data, 0).unwrap(); // SINGLE_WORD mode
                        data
                    }),
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
    ]
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
            for (i, arg) in cmd.args.iter().enumerate().rev() {
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
    write_varint(&mut data, nodes.len() as i32)?;

    // Encode each node
    for node in &nodes {
        data.put_u8(node.flags);

        // Children count and indices
        write_varint(&mut data, node.children.len() as i32)?;
        for &child in &node.children {
            write_varint(&mut data, child)?;
        }

        // Redirect (optional, not used)
        if node.redirect.is_some() {
            write_varint(&mut data, node.redirect.unwrap())?;
        }

        // Name (for literal and argument nodes)
        if let Some(ref name) = node.name {
            name.clone().encode(&mut data)?;
        }

        // Parser (for argument nodes)
        if let Some(parser_id) = node.parser_id {
            write_varint(&mut data, parser_id)?;
            if let Some(ref parser_data) = node.parser_data {
                data.extend_from_slice(parser_data);
            }
        }
    }

    // Root index
    write_varint(&mut data, 0)?;

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
            let count = world.entity_count();
            let mut msg = format!("§aEntities: §f{}\n", count);

            // List players
            let players: Vec<_> = world
                .query::<Name>()
                .filter_map(|(e, name)| {
                    let eid = world.get::<EntityId>(e)?;
                    Some(format!("  §7- §f{} §7(id: {})", name.value, eid.value))
                })
                .collect();

            if !players.is_empty() {
                msg.push_str("§aPlayers:\n");
                for p in players {
                    msg.push_str(&p);
                    msg.push('\n');
                }
            }
            Ok(msg.trim_end().to_string())
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
                return Err("Usage: /history <entity> <component>".to_string());
            }
            // TODO: Implement component history tracking
            Err("§cHistory tracking not yet implemented".to_string())
        }
        _ => Err(format!("§cUnknown command: /{}", cmd)),
    }
}

/// System: Handle incoming chat commands
pub fn system_handle_commands(world: &mut World) {
    let play_entities: Vec<_> = world
        .query::<InPlayState>()
        .map(|(entity, _)| entity)
        .collect();

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
                .map(|n| n.value.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            info!("{} executed command: /{}", executor_name, command_str);

            if let Some((cmd, args)) = parse_command(&command_str) {
                let response = match execute_command(cmd, &args, executor, world) {
                    Ok(msg) => msg,
                    Err(err) => err,
                };
                send_chat_message(&mut buffer, &response);
            }
        }

        world.update(executor, buffer);
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
