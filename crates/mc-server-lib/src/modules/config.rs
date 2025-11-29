use flecs_ecs::prelude::*;
use mc_protocol::Decode;
use tracing::{debug, info};

use crate::components::{Connection, ConnectionState, NeedsSpawnChunks, PacketBuffer, ProtocolState};
use crate::packets::encode_packet;
use crate::world_gen::{
    create_biome_registry, create_cat_variant_registry, create_chicken_variant_registry,
    create_cow_variant_registry, create_damage_type_registry, create_dimension_type_registry,
    create_frog_variant_registry, create_painting_variant_registry, create_pig_variant_registry,
    create_wolf_sound_variant_registry, create_wolf_variant_registry,
    create_zombie_nautilus_variant_registry,
};

/// Configuration module - handles configuration phase
#[derive(Component)]
pub struct ConfigurationModule;

impl Module for ConfigurationModule {
    fn module(world: &World) {
        world.module::<ConfigurationModule>("configuration");

        // Register components
        world.component::<NeedsSpawnChunks>();

        // Handle configuration packets
        world
            .system_named::<(&mut ProtocolState, &mut PacketBuffer)>("HandleConfiguration")
            .with(Connection)
            .each_entity(|e, (state, buffer)| {
                if state.0 != ConnectionState::Configuration {
                    return;
                }

                while let Some((packet_id, data)) = buffer.pop_incoming() {
                    match packet_id {
                        0 => {
                            // Client Information
                            debug!("Got Client Information");
                        }
                        2 => {
                            // Custom Payload (plugin message)
                            let mut cursor = std::io::Cursor::new(&data[..]);
                            if let Ok(channel) = String::decode(&mut cursor) {
                                debug!("Plugin message on channel: {}", channel);
                            }
                        }
                        3 => {
                            // Finish Configuration (Acknowledge)
                            info!("Client acknowledged configuration, transitioning to Play");
                            state.0 = ConnectionState::Play;
                            e.add(NeedsSpawnChunks);
                        }
                        7 => {
                            // Select Known Packs response
                            debug!("Client selected known packs");

                            // Send Registry Data
                            send_registry_data(buffer);

                            // Send Finish Configuration
                            let packet = encode_packet(3, &[]);
                            buffer.push_outgoing(packet);
                            debug!("Sent Finish Configuration");
                        }
                        _ => {
                            debug!("Unknown configuration packet: {}", packet_id);
                        }
                    }
                }
            });
    }
}

fn send_registry(buffer: &mut PacketBuffer, data: Vec<u8>) {
    let packet = encode_packet(7, &data);
    buffer.push_outgoing(packet);
}

fn send_registry_data(buffer: &mut PacketBuffer) {
    if let Ok(data) = create_dimension_type_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_biome_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_damage_type_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_cat_variant_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_chicken_variant_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_cow_variant_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_frog_variant_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_pig_variant_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_wolf_variant_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_wolf_sound_variant_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_zombie_nautilus_variant_registry() { send_registry(buffer, data); }
    if let Ok(data) = create_painting_variant_registry() { send_registry(buffer, data); }

    debug!("Sent all registry data");
}
