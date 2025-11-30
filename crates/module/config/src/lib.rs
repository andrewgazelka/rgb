//! Configuration module - handles configuration phase

mod registry;

use bytes::{BufMut, Bytes, BytesMut};
use flecs_ecs::prelude::*;
use mc_protocol::{write_varint, Decode};
use module_login::NeedsSpawnChunks;
use module_loader::register_module;
use module_network_components::{Connection, ConnectionState, PacketBuffer, ProtocolState};
use registry::{
    create_biome_registry, create_cat_variant_registry, create_chicken_variant_registry,
    create_cow_variant_registry, create_damage_type_registry, create_dimension_type_registry,
    create_frog_variant_registry, create_painting_variant_registry, create_pig_variant_registry,
    create_wolf_sound_variant_registry, create_wolf_variant_registry,
    create_zombie_nautilus_variant_registry,
};
use tracing::{debug, info};

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

/// Configuration module - handles configuration phase
#[derive(Component)]
pub struct ConfigurationModule;

impl Module for ConfigurationModule {
    fn module(world: &World) {
        world.module::<ConfigurationModule>("configuration");

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
    let mut cursor = std::io::Cursor::new(&data);
    if let Ok(name) = <String as Decode>::decode(&mut cursor) {
        debug!("Sending registry: {} ({} bytes)", name, data.len());
    }
    let packet = encode_packet(7, &data);
    buffer.push_outgoing(packet);
}

fn send_registry_data(buffer: &mut PacketBuffer) {
    if let Ok(data) = create_dimension_type_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_biome_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_damage_type_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_cat_variant_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_chicken_variant_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_cow_variant_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_frog_variant_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_pig_variant_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_wolf_variant_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_wolf_sound_variant_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_zombie_nautilus_variant_registry() {
        send_registry(buffer, data);
    }
    if let Ok(data) = create_painting_variant_registry() {
        send_registry(buffer, data);
    }

    debug!("Sent all registry data");
}

register_module! {
    name: "config",
    version: 1,
    module: ConfigurationModule,
    path: "::configuration",
}
