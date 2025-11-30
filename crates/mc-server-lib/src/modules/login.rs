use flecs_ecs::prelude::*;
use tracing::{debug, info};

use crate::EntityIdCounter;
use crate::components::{
    ChunkPosition, Connection, ConnectionState, EntityId, GameMode, Name, PacketBuffer, Player,
    Position, ProtocolState, Rotation, Uuid, register_uuid_meta,
};
use crate::packets::{
    create_known_packs, create_login_success, encode_packet, offline_uuid, parse_login_start,
};

fn try_parse_login(data: &[u8]) -> Option<(String, u128)> {
    parse_login_start(data).ok()
}

fn send_login_success(buffer: &mut PacketBuffer, uuid: u128, name: &str) {
    let result: eyre::Result<Vec<u8>> = create_login_success(uuid, name);
    if let Ok(response_data) = result {
        let packet = encode_packet(2, &response_data);
        buffer.push_outgoing(packet);
    }
}

fn send_known_packs(buffer: &mut PacketBuffer) {
    let result: eyre::Result<Vec<u8>> = create_known_packs();
    if let Ok(data) = result {
        let packet = encode_packet(14, &data);
        buffer.push_outgoing(packet);
    }
}

/// Login module - handles login flow
#[derive(Component)]
pub struct LoginModule;

impl Module for LoginModule {
    fn module(world: &World) {
        world.module::<LoginModule>("login");

        // Register components
        world.component::<Player>();
        world.component::<Name>();
        world.component::<Uuid>();
        register_uuid_meta(world); // Custom opaque serialization for u128
        world.component::<EntityId>();
        world.component::<Position>();
        world.component::<Rotation>();
        world.component::<ChunkPosition>();
        world.component::<GameMode>();

        // Set up EntityIdCounter singleton
        world
            .component::<EntityIdCounter>()
            .add_trait::<flecs::Singleton>();
        world.set(EntityIdCounter::default());

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

                debug!("HandleLogin: checking for packets (state={:?})", state.0);

                while let Some((packet_id, data)) = buffer.pop_incoming() {
                    debug!("HandleLogin: got packet_id={}", packet_id);
                    match packet_id {
                        0 => {
                            // Login Start
                            if let Some((name, _uuid)) = try_parse_login(&data) {
                                let player_uuid = offline_uuid(&name);
                                info!("Login from: {} (uuid: {:032x})", &name, player_uuid);

                                // Rename entity to use player hierarchy
                                let player_path = format!("players::{}", name);
                                e.set_name(&player_path);

                                // Add player components
                                let entity_id = entity_counter.next();
                                e.add(Player);
                                e.set(Name {
                                    value: name.clone(),
                                });
                                e.set(Uuid(player_uuid));
                                e.set(EntityId { value: entity_id });
                                e.set(Position::new(0.0, 4.0, 0.0));
                                e.set(Rotation::new(0.0, 0.0));
                                e.set(ChunkPosition::new(0, 0));
                                e.set(GameMode::CREATIVE);

                                // Send Login Success
                                send_login_success(buffer, player_uuid, &name);
                                info!("Sent Login Success, waiting for Login Acknowledged");
                            }
                        }
                        3 => {
                            // Login Acknowledged
                            info!("Login Acknowledged, transitioning to Configuration");
                            state.0 = ConnectionState::Configuration;

                            // Send Known Packs
                            send_known_packs(buffer);
                            debug!("Sent Known Packs");
                        }
                        _ => {
                            debug!("Unknown login packet: {}", packet_id);
                        }
                    }
                }
            });
    }
}
