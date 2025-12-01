//! Login system

use rgb_ecs::{Entity, World};
use tracing::{debug, info};

use crate::components::{
    ChunkPosition, ConnectionState, EntityId, EntityIdCounter, GameMode, Name, PacketBuffer,
    Player, Position, ProtocolState, Rotation, Uuid,
};
use crate::protocol::{offline_uuid, parse_login_start, send_known_packs, send_login_success};

/// System: Handle login packets
pub fn system_handle_login(world: &mut World) {
    let Some(entity_counter) = world.get::<EntityIdCounter>(Entity::WORLD) else {
        return;
    };

    // Query all entities in Login state
    let login_entities: Vec<_> = world
        .query::<ProtocolState>()
        .filter(|(_, state)| state.0 == ConnectionState::Login)
        .map(|(entity, _)| entity)
        .collect();

    for entity in login_entities {
        let Some(mut buffer) = world.get::<PacketBuffer>(entity) else {
            continue;
        };

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
                        world.insert(entity, Player);
                        world.insert(
                            entity,
                            Name {
                                value: name.clone(),
                            },
                        );
                        world.insert(entity, Uuid(player_uuid));
                        world.insert(
                            entity,
                            EntityId {
                                value: new_entity_id,
                            },
                        );
                        world.insert(entity, Position::SPAWN);
                        world.insert(entity, Rotation::new(0.0, 0.0));
                        world.insert(entity, ChunkPosition::new(0, 0));
                        world.insert(entity, GameMode::CREATIVE);

                        send_login_success(&mut buffer, player_uuid, &name);
                        info!("Sent Login Success, waiting for Login Acknowledged");
                    }
                }
                3 => {
                    // Login Acknowledged
                    info!("Login Acknowledged, transitioning to Configuration");
                    world.update(entity, ProtocolState(ConnectionState::Configuration));
                    send_known_packs(&mut buffer);
                    debug!("Sent Known Packs");
                }
                _ => {
                    debug!("Unknown login packet: {}", packet_id);
                }
            }
        }

        world.update(entity, buffer);
    }
}
