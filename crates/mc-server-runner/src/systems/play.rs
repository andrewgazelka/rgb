//! Play state systems

use bytes::Bytes;
use flecs_ecs::prelude::*;
use mc_protocol::Decode;
use tracing::debug;

use crate::components::{
    ChunkData, ChunkPos, EntityId, InPlayState, NeedsSpawnChunks, PacketBuffer, Position, Rotation,
    ServerConfig, TpsTracker, WorldTime,
};
use crate::protocol::{
    send_action_bar, send_chunks_to_buffer, send_game_event_start_waiting, send_keepalive,
    send_play_login, send_player_position, send_set_center_chunk, send_set_time,
};
use crate::systems::send_commands_to_player;

/// System: Send spawn data to new players
pub fn system_send_spawn_data<T>(it: &TableIter<false, T>) {
    let world = it.world();

    // Get singletons
    let config = world.get::<&ServerConfig>(|c| c.clone());
    let world_time = world.get::<&WorldTime>(|t| *t);

    for i in it.iter() {
        let entity = it.entity(i);

        entity.try_get::<(&mut PacketBuffer, &Position, &EntityId)>(|(buffer, pos, entity_id)| {
            send_play_login(buffer, entity_id.value, config.max_players);
            send_game_event_start_waiting(buffer);

            let (cx, cz) = pos.chunk_pos();
            send_set_center_chunk(buffer, cx, cz);

            let chunks = collect_chunks_for_player(8, &world);
            send_chunks_to_buffer(buffer, &chunks);

            send_set_time(buffer, world_time.world_age, world_time.time_of_day);
            send_player_position(buffer, pos.x, pos.y, pos.z, 1);
            send_keepalive(buffer);

            // Send command tree for tab completion
            send_commands_to_player(buffer);

            tracing::info!("Player entered play state");
        });

        // Remove NeedsSpawnChunks and add InPlayState
        entity.remove::<NeedsSpawnChunks>();
        entity.add::<InPlayState>();
    }
}

/// Handle movement for a single entity
pub fn handle_movement(buffer: &mut PacketBuffer, pos: &mut Position, rot: &mut Rotation) {
    while let Some((packet_id, data)) = buffer.pop_incoming() {
        let mut cursor = std::io::Cursor::new(&data[..]);
        match packet_id {
            0x1D => {
                // MovePlayerPos
                if let (Ok(x), Ok(y), Ok(z)) = (
                    f64::decode(&mut cursor),
                    f64::decode(&mut cursor),
                    f64::decode(&mut cursor),
                ) {
                    pos.x = x;
                    pos.y = y;
                    pos.z = z;
                }
            }
            0x1E => {
                // MovePlayerPosRot
                if let (Ok(x), Ok(y), Ok(z), Ok(yaw), Ok(pitch)) = (
                    f64::decode(&mut cursor),
                    f64::decode(&mut cursor),
                    f64::decode(&mut cursor),
                    f32::decode(&mut cursor),
                    f32::decode(&mut cursor),
                ) {
                    pos.x = x;
                    pos.y = y;
                    pos.z = z;
                    rot.yaw = yaw;
                    rot.pitch = pitch;
                }
            }
            0x1F => {
                // MovePlayerRot
                if let (Ok(yaw), Ok(pitch)) = (f32::decode(&mut cursor), f32::decode(&mut cursor)) {
                    rot.yaw = yaw;
                    rot.pitch = pitch;
                }
            }
            0x20 => {
                // MovePlayerStatusOnly - just on_ground, ignore
            }
            0x00 => {
                // AcceptTeleportation
                if let Ok(teleport_id) = mc_protocol::read_varint(&mut cursor) {
                    debug!("Client accepted teleport: {}", teleport_id);
                }
            }
            0x1A => {
                // KeepAlive response
                if let Ok(ka_id) = i64::decode(&mut cursor) {
                    debug!("Keep alive response: {}", ka_id);
                }
            }
            _ => {
                // Unknown packet, put it back
                buffer.push_incoming(packet_id, Bytes::from(data.to_vec()));
                break;
            }
        }
    }
}

/// System: Send periodic keepalive
pub fn system_send_keepalive<T>(it: &TableIter<false, T>) {
    let world = it.world();
    let world_time = world.get::<&WorldTime>(|t| *t);

    // Only send every 300 ticks (15 seconds at 20 TPS)
    if world_time.world_age % 300 != 0 {
        return;
    }

    for i in it.iter() {
        let entity = it.entity(i);
        entity.try_get::<&mut PacketBuffer>(|buffer| {
            send_keepalive(buffer);
        });
    }
}

/// System: Send position and TPS to action bar
pub fn system_send_position_action_bar<T>(it: &TableIter<false, T>) {
    let world = it.world();
    let world_time = world.get::<&WorldTime>(|t| *t);
    let tps = world.get::<&TpsTracker>(|t| *t);

    // Only send every 10 ticks (0.5 seconds at 20 TPS)
    if world_time.world_age % 10 != 0 {
        return;
    }

    for i in it.iter() {
        let entity = it.entity(i);
        entity.try_get::<(&mut PacketBuffer, &Position)>(|(buffer, pos)| {
            let text = format!(
                "X: {:.1} Y: {:.1} Z: {:.1} | TPS: {:.1}:{:.1}:{:.1}",
                pos.x, pos.y, pos.z, tps.tps_5s, tps.tps_15s, tps.tps_1m
            );
            send_action_bar(buffer, &text);
        });
    }
}

fn collect_chunks_for_player(view_distance: i32, world: &WorldRef) -> Vec<Bytes> {
    let mut chunks = Vec::new();

    for cx in -view_distance..=view_distance {
        for cz in -view_distance..=view_distance {
            let pos = ChunkPos::new(cx, cz);
            let name = format!("chunk:{}:{}", pos.x, pos.z);
            if let Some(chunk_entity) = world.try_lookup_recursive(&name) {
                chunk_entity.try_get::<&ChunkData>(|chunk_data| {
                    chunks.push(Bytes::clone(&chunk_data.encoded));
                });
            }
        }
    }

    chunks
}
