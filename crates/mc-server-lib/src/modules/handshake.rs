use flecs_ecs::prelude::*;
use tracing::info;

use crate::components::{Connection, ConnectionState, PacketBuffer, ProtocolState};
use crate::packets::{create_status_response, encode_packet, parse_handshake};

fn send_status_response(buffer: &mut PacketBuffer) {
    if let Ok(response_data) = create_status_response() {
        let packet = encode_packet(0, &response_data);
        buffer.push_outgoing(packet);
    }
}

/// Handshake module - handles initial connection handshake
#[derive(Component)]
pub struct HandshakeModule;

impl Module for HandshakeModule {
    fn module(world: &World) {
        world.module::<HandshakeModule>("handshake");

        // Register components
        world.component::<ProtocolState>();

        // Handle handshake packets
        world
            .system_named::<(&mut ProtocolState, &mut PacketBuffer)>("HandleHandshake")
            .with(Connection)
            .each(|(state, buffer)| {
                if state.0 != ConnectionState::Handshaking {
                    return;
                }

                if let Some((packet_id, data)) = buffer.pop_incoming() {
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
                            if let Ok(response_data) = create_status_response() {
                                let packet = encode_packet(0, &response_data);
                                buffer.push_outgoing(packet);
                            }
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
