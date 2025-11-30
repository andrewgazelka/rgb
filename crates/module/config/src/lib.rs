//! Configuration module - handles configuration phase

use flecs_ecs::prelude::*;
use mc_protocol::Decode;
use mc_server_lib::{
    Connection, ConnectionState, NeedsSpawnChunks, PacketBuffer, ProtocolState,
    create_biome_registry, create_cat_variant_registry, create_chicken_variant_registry,
    create_cow_variant_registry, create_damage_type_registry, create_dimension_type_registry,
    create_frog_variant_registry, create_painting_variant_registry, create_pig_variant_registry,
    create_wolf_sound_variant_registry, create_wolf_variant_registry,
    create_zombie_nautilus_variant_registry, encode_packet,
};
use module_loader::register_plugin;
use tracing::{debug, info};

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
    // Extract registry name for debugging (it's the first string in the data)
    let mut cursor = std::io::Cursor::new(&data);
    if let Ok(name) = <String as mc_protocol::Decode>::decode(&mut cursor) {
        debug!("Sending registry: {} ({} bytes)", name, data.len());
    }
    let packet = encode_packet(7, &data);
    buffer.push_outgoing(packet);
}

fn send_registry_data(buffer: &mut PacketBuffer) {
    match create_dimension_type_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create dimension_type registry: {}", e),
    }
    match create_biome_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create biome registry: {}", e),
    }
    match create_damage_type_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create damage_type registry: {}", e),
    }
    match create_cat_variant_registry() {
        Ok(data) => {
            debug!("Sending cat_variant registry with {} bytes", data.len());
            send_registry(buffer, data);
        }
        Err(e) => tracing::error!("Failed to create cat_variant registry: {}", e),
    }
    match create_chicken_variant_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create chicken_variant registry: {}", e),
    }
    match create_cow_variant_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create cow_variant registry: {}", e),
    }
    match create_frog_variant_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create frog_variant registry: {}", e),
    }
    match create_pig_variant_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create pig_variant registry: {}", e),
    }
    match create_wolf_variant_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create wolf_variant registry: {}", e),
    }
    match create_wolf_sound_variant_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create wolf_sound_variant registry: {}", e),
    }
    match create_zombie_nautilus_variant_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create zombie_nautilus_variant registry: {}", e),
    }
    match create_painting_variant_registry() {
        Ok(data) => send_registry(buffer, data),
        Err(e) => tracing::error!("Failed to create painting_variant registry: {}", e),
    }

    debug!("Sent all registry data");
}

register_plugin! {
    name: "config",
    version: 1,
    module: ConfigurationModule,
    path: "::configuration",
}
