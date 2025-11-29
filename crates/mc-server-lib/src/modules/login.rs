use flecs_ecs::prelude::*;
use tracing::{debug, info};

use crate::components::{
    ChunkPosition, Connection, ConnectionState, EntityId, GameMode, PacketBuffer, Player,
    PlayerName, PlayerUuid, Position, ProtocolState, Rotation,
};
use crate::packets::{create_known_packs, create_login_success, encode_packet, offline_uuid, parse_login_start};
use crate::EntityIdCounter;

/// Login module - handles login flow
#[derive(Component)]
pub struct LoginModule;

impl Module for LoginModule {
    fn module(world: &World) {
        world.module::<LoginModule>("login");

        // Register components
        world.component::<Player>();
        world.component::<PlayerName>();
        world.component::<PlayerUuid>();
        world.component::<EntityId>();
        world.component::<Position>();
        world.component::<Rotation>();
        world.component::<ChunkPosition>();
        world.component::<GameMode>();

        // Register EntityIdCounter singleton
        world
            .component::<EntityIdCounter>()
            .add_trait::<flecs::Singleton>();

        // Handle login packets
        world
            .system_named::<(&mut ProtocolState, &mut PacketBuffer, &EntityIdCounter)>(
                "HandleLogin",
            )
            .with(Connection)
            .each_entity(|e, (state, buffer, entity_counter)| {
                if state.0 != ConnectionState::Login {
                    return;
                }

                while let Some((packet_id, data)) = buffer.pop_incoming() {
                    match packet_id {
                        0 => {
                            // Login Start
                            if let Some((name, _uuid)) = parse_login_start(&data).ok() {
                                let player_uuid = offline_uuid(&name);
                                info!("Login from: {} (uuid: {:032x})", &name, player_uuid);

                                // Add player components
                                let entity_id = entity_counter.next();
                                e.add(Player);
                                e.set(PlayerName(name.clone()));
                                e.set(PlayerUuid(player_uuid));
                                e.set(EntityId(entity_id));
                                e.set(Position::new(0.0, 4.0, 0.0));
                                e.set(Rotation::new(0.0, 0.0));
                                e.set(ChunkPosition::new(0, 0));
                                e.set(GameMode::CREATIVE);

                                // Send Login Success
                                if let Some(response_data) = create_login_success(player_uuid, &name).ok() {
                                    let packet = encode_packet(2, &response_data);
                                    buffer.push_outgoing(packet);
                                    info!("Sent Login Success, waiting for Login Acknowledged");
                                }
                            }
                        }
                        3 => {
                            // Login Acknowledged
                            info!("Login Acknowledged, transitioning to Configuration");
                            state.0 = ConnectionState::Configuration;

                            // Send Known Packs
                            if let Some(data) = create_known_packs().ok() {
                                let packet = encode_packet(14, &data);
                                buffer.push_outgoing(packet);
                                debug!("Sent Known Packs");
                            }
                        }
                        _ => {
                            debug!("Unknown login packet: {}", packet_id);
                        }
                    }
                }
            });
    }
}
