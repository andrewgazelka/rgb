//! Command system
//!
//! Handles incoming chat commands and generates Minecraft command tree packets.

use bytes::{BufMut, Bytes, BytesMut};
use flecs_ecs::prelude::*;
use mc_protocol::{Decode, Encode};
use tracing::{debug, info};

use crate::components::{
    EntityId, InPlayState, Name, PacketBuffer, Position, Rotation, TpsTracker,
};
use crate::protocol::encode_packet;

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
    pub const STRING_SINGLE_WORD: i32 = 5;
    pub const ENTITY: i32 = 6;
}

/// Command definition for building command trees
#[derive(Clone)]
pub struct CommandDef {
    pub name: &'static str,
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
            args: vec![ArgDef {
                name: "entity",
                parser_id: parser_ids::ENTITY,
                parser_data: Some(vec![0x01]),
            }],
        },
        CommandDef {
            name: "tps",
            args: vec![],
        },
        CommandDef {
            name: "pos",
            args: vec![],
        },
        CommandDef {
            name: "entities",
            args: vec![],
        },
    ]
}

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

    nodes.push(CommandNode {
        flags: NODE_TYPE_ROOT,
        children: Vec::new(),
        redirect: None,
        name: None,
        parser_id: None,
        parser_data: None,
    });

    let mut root_children = Vec::new();

    for cmd in &commands {
        let cmd_node_idx = nodes.len() as i32;
        root_children.push(cmd_node_idx);

        if cmd.args.is_empty() {
            nodes.push(CommandNode {
                flags: NODE_TYPE_LITERAL | FLAG_EXECUTABLE,
                children: Vec::new(),
                redirect: None,
                name: Some(cmd.name.to_string()),
                parser_id: None,
                parser_data: None,
            });
        } else {
            let mut arg_indices = Vec::new();
            for (i, _arg) in cmd.args.iter().enumerate().rev() {
                let arg_idx = nodes.len() as i32 + (cmd.args.len() - 1 - i) as i32 + 1;
                arg_indices.push(arg_idx);
            }
            arg_indices.reverse();

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

    nodes[0].children = root_children;

    let mut data = BytesMut::new();
    write_varint_bytes(&mut data, nodes.len() as i32);

    for node in &nodes {
        data.put_u8(node.flags);
        write_varint_bytes(&mut data, node.children.len() as i32);
        for &child in &node.children {
            write_varint_bytes(&mut data, child);
        }

        if node.redirect.is_some() {
            write_varint_bytes(&mut data, node.redirect.unwrap());
        }

        if let Some(ref name) = node.name {
            let mut name_buf = Vec::new();
            name.clone().encode(&mut name_buf)?;
            data.extend_from_slice(&name_buf);
        }

        if let Some(parser_id) = node.parser_id {
            write_varint_bytes(&mut data, parser_id);
            if let Some(ref parser_data) = node.parser_data {
                data.extend_from_slice(parser_data);
            }
        }
    }

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

fn send_chat_message(buffer: &mut PacketBuffer, message: &str) {
    let mut data = BytesMut::new();
    let nbt = mc_protocol::nbt! {
        "text" => message,
    };
    data.extend_from_slice(&nbt.to_network_bytes());
    data.put_u8(0);
    buffer.push_outgoing(encode_packet(SYSTEM_CHAT_PACKET_ID, &data));
}

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

fn execute_command(
    cmd: &str,
    args: &[&str],
    executor: EntityView<'_>,
    world: &WorldRef<'_>,
) -> Result<String, String> {
    match cmd {
        "tps" => {
            let tps = world.get::<&TpsTracker>(|t| *t);
            Ok(format!(
                "TPS: {:.1} (5s) {:.1} (15s) {:.1} (1m)",
                tps.tps_5s, tps.tps_15s, tps.tps_1m
            ))
        }
        "pos" => {
            let pos = executor
                .try_get::<&Position>(|p| *p)
                .ok_or("Position not found")?;
            let rot = executor.try_get::<&Rotation>(|r| *r);
            if let Some(rot) = rot {
                Ok(format!(
                    "Position: {:.2}, {:.2}, {:.2} | Yaw: {:.1} Pitch: {:.1}",
                    pos.x, pos.y, pos.z, rot.yaw, rot.pitch
                ))
            } else {
                Ok(format!(
                    "Position: {:.2}, {:.2}, {:.2}",
                    pos.x, pos.y, pos.z
                ))
            }
        }
        "entities" => {
            let mut count = 0;
            world.query::<&EntityId>().build().each(|_| count += 1);
            Ok(format!("Total entities with EntityId: {}", count))
        }
        "inspect" => {
            if args.is_empty() {
                return Err("Usage: /inspect <entity>".to_string());
            }

            if args[0] == "@s" {
                let mut components = Vec::new();

                if let Some(name) = executor.try_get::<&Name>(|n| n.value.clone()) {
                    components.push(format!("Name: {}", name));
                }
                if let Some(pos) = executor.try_get::<&Position>(|p| *p) {
                    components.push(format!(
                        "Position: {:.2}, {:.2}, {:.2}",
                        pos.x, pos.y, pos.z
                    ));
                }
                if let Some(rot) = executor.try_get::<&Rotation>(|r| *r) {
                    components.push(format!(
                        "Rotation: yaw={:.1} pitch={:.1}",
                        rot.yaw, rot.pitch
                    ));
                }
                if let Some(eid) = executor.try_get::<&EntityId>(|e| e.value) {
                    components.push(format!("EntityId: {}", eid));
                }
                if executor.has(InPlayState) {
                    components.push("InPlayState: true".to_string());
                }

                if components.is_empty() {
                    Ok("No known components found".to_string())
                } else {
                    Ok(format!("Components:\n{}", components.join("\n")))
                }
            } else {
                Err("Invalid entity selector. Use @s".to_string())
            }
        }
        _ => Err(format!("Unknown command: /{}", cmd)),
    }
}

/// Handle incoming chat commands
pub fn handle_commands(world: &WorldRef<'_>, executor: EntityView<'_>, buffer: &mut PacketBuffer) {
    let mut commands_to_execute = Vec::new();
    let mut remaining = Vec::new();

    while let Some((packet_id, data)) = buffer.pop_incoming() {
        if packet_id == CHAT_COMMAND_PACKET_ID {
            let mut cursor = std::io::Cursor::new(&data[..]);
            if let Ok(command_str) = String::decode(&mut cursor) {
                commands_to_execute.push(command_str);
            }
        } else {
            remaining.push((packet_id, data));
        }
    }

    for (id, data) in remaining {
        buffer.push_incoming(id, data);
    }

    for command_str in commands_to_execute {
        let executor_name = executor
            .try_get::<&Name>(|n| n.value.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        info!("{} executed command: /{}", executor_name, command_str);

        if let Some((cmd, args)) = parse_command(&command_str) {
            let response = match execute_command(cmd, &args, executor, world) {
                Ok(msg) => msg,
                Err(err) => err,
            };
            send_chat_message(buffer, &response);
        }
    }
}

/// Send command tree to new players
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
