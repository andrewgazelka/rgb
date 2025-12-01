//! Play state systems

use bytes::Bytes;
use mc_protocol::Decode;
use rgb_ecs::{Entity, World};
use tracing::debug;

use crate::components::{
    ChunkData, ChunkIndex, ChunkPos, EntityId, InPlayState, NeedsSpawnChunks, PacketBuffer,
    Position, Rotation, WorldTime,
};
use crate::protocol::{
    send_action_bar, send_chunks_to_buffer, send_game_event_start_waiting, send_keepalive,
    send_play_login, send_player_position, send_set_center_chunk, send_set_time,
};

/// System: Send spawn data to new players
pub fn system_send_spawn_data(world: &mut World) {
    let Some(chunk_index) = world.get::<ChunkIndex>(Entity::WORLD) else {
        return;
    };
    let Some(world_time) = world.get::<WorldTime>(Entity::WORLD) else {
        return;
    };

    // Query entities that need spawn chunks
    let need_spawn: Vec<_> = world
        .query::<NeedsSpawnChunks>()
        .map(|(entity, _)| entity)
        .collect();

    for entity in need_spawn {
        let Some(mut buffer) = world.get::<PacketBuffer>(entity) else {
            continue;
        };
        let Some(pos) = world.get::<Position>(entity) else {
            continue;
        };
        let Some(entity_id) = world.get::<EntityId>(entity) else {
            continue;
        };

        send_play_login(&mut buffer, entity_id.value);
        send_game_event_start_waiting(&mut buffer);

        let (cx, cz) = pos.chunk_pos();
        send_set_center_chunk(&mut buffer, cx, cz);

        let chunks = collect_chunks_for_player(&chunk_index, 8, world);
        send_chunks_to_buffer(&mut buffer, &chunks);

        send_set_time(&mut buffer, world_time.world_age, world_time.time_of_day);
        send_player_position(&mut buffer, pos.x, pos.y, pos.z, 1);
        send_keepalive(&mut buffer);

        // Remove NeedsSpawnChunks and add InPlayState
        world.remove::<NeedsSpawnChunks>(entity);
        world.insert(entity, InPlayState);
        world.update(entity, buffer);

        tracing::info!("Player entered play state");
    }
}

/// System: Handle player movement packets
pub fn system_handle_movement(world: &mut World) {
    // Query all players in Play state who are InPlayState
    let play_entities: Vec<_> = world
        .query::<InPlayState>()
        .map(|(entity, _)| entity)
        .collect();

    for entity in play_entities {
        let Some(mut buffer) = world.get::<PacketBuffer>(entity) else {
            continue;
        };
        let Some(mut pos) = world.get::<Position>(entity) else {
            continue;
        };
        let Some(mut rot) = world.get::<Rotation>(entity) else {
            continue;
        };

        let mut pos_changed = false;
        let mut rot_changed = false;

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
                        pos_changed = true;
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
                        pos_changed = true;
                        rot_changed = true;
                    }
                }
                0x1F => {
                    // MovePlayerRot
                    if let (Ok(yaw), Ok(pitch)) =
                        (f32::decode(&mut cursor), f32::decode(&mut cursor))
                    {
                        rot.yaw = yaw;
                        rot.pitch = pitch;
                        rot_changed = true;
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

        world.update(entity, buffer);
        if pos_changed {
            world.update(entity, pos);
        }
        if rot_changed {
            world.update(entity, rot);
        }
    }
}

/// System: Send periodic keepalive
pub fn system_send_keepalive(world: &mut World) {
    let Some(world_time) = world.get::<WorldTime>(Entity::WORLD) else {
        return;
    };

    // Only send every 300 ticks (15 seconds at 20 TPS)
    if world_time.world_age % 300 != 0 {
        return;
    }

    // Query all players in play state
    let play_entities: Vec<_> = world
        .query::<InPlayState>()
        .map(|(entity, _)| entity)
        .collect();

    for entity in play_entities {
        let Some(mut buffer) = world.get::<PacketBuffer>(entity) else {
            continue;
        };

        send_keepalive(&mut buffer);
        world.update(entity, buffer);
    }
}

/// System: Send position and TPS to action bar
pub fn system_send_position_action_bar(world: &mut World) {
    let Some(world_time) = world.get::<WorldTime>(Entity::WORLD) else {
        return;
    };
    let Some(tps) = world.get::<crate::components::TpsTracker>(Entity::WORLD) else {
        return;
    };

    // Only send every 10 ticks (0.5 seconds at 20 TPS)
    if world_time.world_age % 10 != 0 {
        return;
    }

    // Query all players in play state
    let play_entities: Vec<_> = world
        .query::<InPlayState>()
        .map(|(entity, _)| entity)
        .collect();

    for entity in play_entities {
        let Some(mut buffer) = world.get::<PacketBuffer>(entity) else {
            continue;
        };
        let Some(pos) = world.get::<Position>(entity) else {
            continue;
        };

        let text = format!(
            "X: {:.1} Y: {:.1} Z: {:.1} | TPS: {:.1}:{:.1}:{:.1}",
            pos.x, pos.y, pos.z, tps.tps_5s, tps.tps_15s, tps.tps_1m
        );
        send_action_bar(&mut buffer, &text);
        world.update(entity, buffer);
    }
}

fn collect_chunks_for_player(
    chunk_index: &ChunkIndex,
    view_distance: i32,
    world: &World,
) -> Vec<Bytes> {
    let mut chunks = Vec::new();

    for cx in -view_distance..=view_distance {
        for cz in -view_distance..=view_distance {
            let pos = ChunkPos::new(cx, cz);
            if let Some(chunk_entity) = chunk_index.get(&pos) {
                if let Some(chunk_data) = world.get::<ChunkData>(chunk_entity) {
                    chunks.push(Bytes::clone(&chunk_data.encoded));
                }
            }
        }
    }

    chunks
}
