//! Configuration phase system

use flecs_ecs::prelude::*;
use mc_protocol::Decode;
use tracing::debug;

use crate::components::{ConnectionState, NeedsSpawnChunks, PacketBuffer, ProtocolState};
use crate::protocol::encode_packet;
use crate::registry::{
    create_biome_registry, create_cat_variant_registry, create_chicken_variant_registry,
    create_cow_variant_registry, create_damage_type_registry, create_dimension_type_registry,
    create_frog_variant_registry, create_painting_variant_registry, create_pig_variant_registry,
    create_wolf_sound_variant_registry, create_wolf_variant_registry,
    create_zombie_nautilus_variant_registry,
};

/// System: Handle configuration packets
pub fn system_handle_configuration<T>(it: &TableIter<false, T>) {
    for i in it.iter() {
        let entity = it.entity(i);

        entity.try_get::<(&mut PacketBuffer, &mut ProtocolState)>(|(buffer, state)| {
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
                        tracing::info!("Client acknowledged configuration, transitioning to Play");
                        state.0 = ConnectionState::Play;
                        entity.add::<NeedsSpawnChunks>();
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
