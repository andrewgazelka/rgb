//! Attack/Combat systems
//!
//! Handles player attacks on entities via the Interact packet (action type = ATTACK).

use flecs_ecs::prelude::*;
use mc_data::play::serverbound::Interact;
use mc_protocol::{Decode, Packet};
use tracing::{debug, info};

use crate::components::{EntityId, InPlayState, Name, PacketBuffer, Position};

/// Interaction action types from the protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum InteractionType {
    Interact = 0,
    Attack = 1,
    InteractAt = 2,
}

impl InteractionType {
    fn from_varint(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Interact),
            1 => Some(Self::Attack),
            2 => Some(Self::InteractAt),
            _ => None,
        }
    }
}

/// Parsed interact packet data
#[derive(Debug, Clone)]
pub struct InteractPacket {
    pub target_entity_id: i32,
    pub action: InteractionType,
    pub sneaking: bool,
}

fn parse_interact_packet(data: &[u8]) -> Option<InteractPacket> {
    let mut cursor = std::io::Cursor::new(data);

    let target_entity_id = mc_protocol::read_varint(&mut cursor).ok()?;
    let action_type = mc_protocol::read_varint(&mut cursor).ok()?;
    let action = InteractionType::from_varint(action_type)?;

    // Skip action-specific data
    match action {
        InteractionType::Interact => {
            // Hand enum (varint)
            let _hand = mc_protocol::read_varint(&mut cursor).ok()?;
        }
        InteractionType::Attack => {
            // No additional data
        }
        InteractionType::InteractAt => {
            // Target position (3 floats) + hand
            let _x = f32::decode(&mut cursor).ok()?;
            let _y = f32::decode(&mut cursor).ok()?;
            let _z = f32::decode(&mut cursor).ok()?;
            let _hand = mc_protocol::read_varint(&mut cursor).ok()?;
        }
    }

    let sneaking = bool::decode(&mut cursor).ok()?;

    Some(InteractPacket {
        target_entity_id,
        action,
        sneaking,
    })
}

/// Serverbound Interact packet ID in Play state
const INTERACT_PACKET_ID: i32 = Interact::ID;

/// System: Handle player attack interactions
pub fn system_handle_attacks<T>(it: &TableIter<false, T>) {
    let world = it.world();

    // Build entity ID to entity map for target lookup
    let mut entity_id_map = std::collections::HashMap::new();
    world
        .query::<&EntityId>()
        .build()
        .each_entity(|entity, eid| {
            entity_id_map.insert(eid.value, entity.id());
        });

    for i in it.iter() {
        let attacker_entity = it.entity(i);

        attacker_entity.try_get::<&mut PacketBuffer>(|buffer| {
            let mut attacks_to_process = Vec::new();

            // Scan for interact packets
            let mut remaining = Vec::new();
            while let Some((packet_id, data)) = buffer.pop_incoming() {
                if packet_id == INTERACT_PACKET_ID {
                    if let Some(interact) = parse_interact_packet(&data) {
                        if interact.action == InteractionType::Attack {
                            attacks_to_process.push(interact);
                        } else {
                            // Non-attack interactions go back for other systems
                            remaining.push((packet_id, data));
                        }
                    }
                } else {
                    remaining.push((packet_id, data));
                }
            }

            // Put remaining packets back
            for (id, data) in remaining {
                buffer.push_incoming(id, data);
            }

            // Process attacks
            for attack in attacks_to_process {
                let attacker_name = attacker_entity
                    .try_get::<&Name>(|n| n.value.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let attacker_pos = attacker_entity.try_get::<&Position>(|p| *p);

                // Find target entity
                if let Some(&target_id) = entity_id_map.get(&attack.target_entity_id) {
                    let target = world.entity_from_id(target_id);
                    let target_name = target
                        .try_get::<&Name>(|n| n.value.clone())
                        .unwrap_or_else(|| format!("Entity#{}", attack.target_entity_id));

                    info!(
                        "{} attacked {} (sneaking: {})",
                        attacker_name, target_name, attack.sneaking
                    );
                } else {
                    debug!(
                        "{} attacked unknown entity ID {} at {:?}",
                        attacker_name, attack.target_entity_id, attacker_pos
                    );
                }
            }
        });
    }
}
