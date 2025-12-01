//! Handshake and status systems

use rgb_ecs::{Entity, World};
use tracing::{debug, info};

use crate::components::{ConnectionIndex, ConnectionState, PacketBuffer, ProtocolState};
use crate::protocol::{encode_packet, parse_handshake, send_status_response};

/// System: Handle handshake packets
pub fn system_handle_handshake(world: &mut World) {
    let Some(conn_index) = world.get::<ConnectionIndex>(Entity::WORLD) else {
        return;
    };

    // Collect entities to process
    let entities: Vec<_> = conn_index.map.values().copied().collect();

    for entity in entities {
        let Some(state) = world.get::<ProtocolState>(entity) else {
            continue;
        };
        if state.0 != ConnectionState::Handshaking {
            continue;
        }

        let Some(mut buffer) = world.get::<PacketBuffer>(entity) else {
            continue;
        };

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

                    let new_state = match next_state {
                        1 => ConnectionState::Status,
                        2 => ConnectionState::Login,
                        _ => {
                            tracing::warn!("Unknown next state: {}", next_state);
                            continue;
                        }
                    };

                    world.update(entity, ProtocolState(new_state));
                }
            }
            world.update(entity, buffer);
        }
    }
}

/// System: Handle status request packets
pub fn system_handle_status(world: &mut World) {
    let Some(conn_index) = world.get::<ConnectionIndex>(Entity::WORLD) else {
        return;
    };

    let entities: Vec<_> = conn_index.map.values().copied().collect();

    for entity in entities {
        let Some(state) = world.get::<ProtocolState>(entity) else {
            continue;
        };
        if state.0 != ConnectionState::Status {
            continue;
        }

        let Some(mut buffer) = world.get::<PacketBuffer>(entity) else {
            continue;
        };

        while let Some((packet_id, data)) = buffer.pop_incoming() {
            match packet_id {
                0 => {
                    // Status Request
                    info!("Status request");
                    send_status_response(&mut buffer);
                }
                1 => {
                    // Ping - echo back the same data
                    let packet = encode_packet(1, &data);
                    buffer.push_outgoing(packet);
                }
                _ => {}
            }
        }

        world.update(entity, buffer);
    }
}
