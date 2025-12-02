//! Login system

use flecs_ecs::prelude::*;
use tracing::{debug, info};

use crate::components::{
    ChunkPosition, ConnectionState, EntityId, EntityIdCounter, GameMode, Name, PacketBuffer,
    Player, Position, ProtocolState, Rotation, Uuid,
};
use crate::protocol::{offline_uuid, parse_login_start, send_known_packs, send_login_success};

/// Handle login packets for a single entity
pub fn handle_login(
    entity: EntityView<'_>,
    buffer: &mut PacketBuffer,
    state: &mut ProtocolState,
    entity_counter: &EntityIdCounter,
) {
    if state.0 != ConnectionState::Login {
        return;
    }

    while let Some((packet_id, data)) = buffer.pop_incoming() {
        debug!("HandleLogin: got packet_id={}", packet_id);
        match packet_id {
            0 => {
                // Login Start
                if let Ok((name, _uuid)) = parse_login_start(&data) {
                    let player_uuid = offline_uuid(&name);
                    info!("Login from: {} (uuid: {:032x})", &name, player_uuid);

                    let new_entity_id = entity_counter.next();

                    // Add player components
                    entity
                        .add(Player)
                        .set(Name {
                            value: name.clone(),
                        })
                        .set(Uuid(player_uuid))
                        .set(EntityId {
                            value: new_entity_id,
                        })
                        .set(Position::SPAWN)
                        .set(Rotation::new(0.0, 0.0))
                        .set(ChunkPosition::new(0, 0))
                        .set(GameMode::CREATIVE);

                    send_login_success(buffer, player_uuid, &name);
                    info!("Sent Login Success, waiting for Login Acknowledged");
                }
            }
            3 => {
                // Login Acknowledged
                info!("Login Acknowledged, transitioning to Configuration");
                state.0 = ConnectionState::Configuration;
                send_known_packs(buffer);
                debug!("Sent Known Packs");
            }
            _ => {
                debug!("Unknown login packet: {}", packet_id);
            }
        }
    }
}
