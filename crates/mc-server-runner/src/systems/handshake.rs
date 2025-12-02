//! Handshake and status systems

use flecs_ecs::prelude::*;
use tracing::{debug, info};

use crate::components::{ConnectionState, PacketBuffer, ProtocolState, ServerConfig};
use crate::protocol::{encode_packet, parse_handshake, send_status_response};

/// Handle handshake for a single entity
pub fn handle_handshake(_entity: EntityView, buffer: &mut PacketBuffer, state: &mut ProtocolState) {
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

                let new_state = match next_state {
                    1 => ConnectionState::Status,
                    2 => ConnectionState::Login,
                    _ => {
                        tracing::warn!("Unknown next state: {}", next_state);
                        return;
                    }
                };

                state.0 = new_state;
            }
        }
    }
}

/// System: Handle status request packets
pub fn system_handle_status<T>(it: &TableIter<false, T>) {
    let world = it.world();

    // Get ServerConfig singleton
    let config = world.get::<&ServerConfig>(|config| config.clone());

    for i in it.iter() {
        let entity = it.entity(i);

        entity.try_get::<(&mut PacketBuffer, &ProtocolState)>(|(buffer, state)| {
            if state.0 != ConnectionState::Status {
                return;
            }

            while let Some((packet_id, data)) = buffer.pop_incoming() {
                match packet_id {
                    0 => {
                        // Status Request
                        info!("Status request");
                        send_status_response(buffer, config.max_players, &config.motd);
                    }
                    1 => {
                        // Ping - echo back the same data
                        let packet = encode_packet(1, &data);
                        buffer.push_outgoing(packet);
                    }
                    _ => {}
                }
            }
        });
    }
}
