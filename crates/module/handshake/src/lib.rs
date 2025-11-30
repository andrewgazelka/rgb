//! Handshake module - handles initial connection handshake and status queries
//!
//! This module provides:
//! - Handshake packet parsing (protocol version, next state)
//! - Status response (server info JSON)
//! - Ping/pong for latency measurement

use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{BufMut, Bytes, BytesMut};
use flecs_ecs::prelude::*;
use mc_protocol::{Decode, Encode, write_varint};
use module_loader::register_plugin;
use module_network_components::{
    Connection, ConnectionState, NetworkComponentsModule, PacketBuffer, ProtocolState,
};
use serde::Serialize;
use tracing::{debug, info};

// ============================================================================
// Packet helpers
// ============================================================================

/// Encode a packet with ID and data into a length-prefixed packet
fn encode_packet(packet_id: i32, data: &[u8]) -> Bytes {
    let mut packet_id_bytes = Vec::new();
    write_varint(&mut packet_id_bytes, packet_id).expect("varint write");

    let length = packet_id_bytes.len() + data.len();
    let mut length_bytes = Vec::new();
    write_varint(&mut length_bytes, length as i32).expect("varint write");

    let mut buf = BytesMut::with_capacity(length_bytes.len() + packet_id_bytes.len() + data.len());
    buf.put_slice(&length_bytes);
    buf.put_slice(&packet_id_bytes);
    buf.put_slice(data);
    buf.freeze()
}

/// Parse a handshake packet, returns (protocol_version, next_state)
fn parse_handshake(data: &[u8]) -> eyre::Result<(i32, i32)> {
    let mut cursor = Cursor::new(data);
    let protocol_version = mc_protocol::read_varint(&mut cursor)?;
    let _server_address = String::decode(&mut cursor)?;
    let _server_port = cursor.read_u16::<BigEndian>()?;
    let next_state = mc_protocol::read_varint(&mut cursor)?;
    Ok((protocol_version, next_state))
}

/// Create status response JSON
fn create_status_response() -> eyre::Result<Vec<u8>> {
    #[derive(Serialize)]
    struct ServerStatus {
        version: Version,
        players: Players,
        description: Description,
        #[serde(rename = "enforcesSecureChat")]
        enforces_secure_chat: bool,
    }

    #[derive(Serialize)]
    struct Version {
        name: String,
        protocol: i32,
    }

    #[derive(Serialize)]
    struct Players {
        max: i32,
        online: i32,
        sample: Vec<PlayerSample>,
    }

    #[derive(Serialize)]
    struct PlayerSample {
        name: String,
        id: String,
    }

    #[derive(Serialize)]
    struct Description {
        text: String,
    }

    let status = ServerStatus {
        version: Version {
            name: mc_data::PROTOCOL_NAME.to_string(),
            protocol: mc_data::PROTOCOL_VERSION,
        },
        players: Players {
            max: 100,
            online: 0,
            sample: vec![],
        },
        description: Description {
            text: "A Rust Minecraft Server (Flecs ECS)".to_string(),
        },
        enforces_secure_chat: false,
    };

    let json = serde_json::to_string(&status)?;
    let mut data = Vec::new();
    json.encode(&mut data)?;
    Ok(data)
}

fn send_status_response(buffer: &mut PacketBuffer) {
    if let Ok(response_data) = create_status_response() {
        let packet = encode_packet(0, &response_data);
        buffer.push_outgoing(packet);
    }
}

// ============================================================================
// Module
// ============================================================================

/// Handshake module - handles initial connection handshake
#[derive(Component)]
pub struct HandshakeModule;

impl Module for HandshakeModule {
    fn module(world: &World) {
        world.module::<HandshakeModule>("handshake");

        // Import network components
        world.import::<NetworkComponentsModule>();

        // Handle handshake packets
        world
            .system_named::<(&mut ProtocolState, &mut PacketBuffer)>("HandleHandshake")
            .with(Connection)
            .each(|(state, buffer)| {
                if state.0 != ConnectionState::Handshaking {
                    return;
                }

                debug!("HandleHandshake: checking for packets");

                if let Some((packet_id, data)) = buffer.pop_incoming() {
                    debug!("HandleHandshake: got packet_id={}", packet_id);
                    if packet_id == 0 {
                        // Handshake packet
                        if let Ok((protocol_version, next_state)) = parse_handshake(&data) {
                            info!(
                                "Handshake: protocol={}, next_state={}",
                                protocol_version, next_state
                            );

                            state.0 = match next_state {
                                1 => ConnectionState::Status,
                                2 => ConnectionState::Login,
                                _ => {
                                    tracing::warn!("Unknown next state: {}", next_state);
                                    return;
                                }
                            };
                        }
                    }
                }
            });

        // Handle status request packets
        world
            .system_named::<(&mut ProtocolState, &mut PacketBuffer)>("HandleStatus")
            .with(Connection)
            .each(|(state, buffer)| {
                if state.0 != ConnectionState::Status {
                    return;
                }

                while let Some((packet_id, data)) = buffer.pop_incoming() {
                    match packet_id {
                        0 => {
                            // Status Request
                            info!("Status request");
                            send_status_response(buffer);
                        }
                        1 => {
                            // Ping - echo back the same data
                            let packet = encode_packet(1, &data);
                            buffer.push_outgoing(packet);
                            // Connection will be closed by async layer after ping
                        }
                        _ => {}
                    }
                }
            });
    }
}

// ============================================================================
// Plugin exports
// ============================================================================

register_module! {
    name: "handshake",
    version: 1,
    module: HandshakeModule,
    path: "::handshake",
}
