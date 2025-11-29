use bytes::Bytes;
use flecs_ecs::prelude::*;
use mc_data::play::clientbound::{
    ChunkBatchFinished, ChunkBatchStart, GameEvent, KeepAlive as ClientboundKeepAlive,
    LevelChunkWithLight, Login as PlayLogin, PlayerPosition, SetChunkCacheCenter, SetTime,
};
use mc_data::play::serverbound::{
    AcceptTeleportation, KeepAlive as ServerboundKeepAlive, MovePlayerPos, MovePlayerPosRot,
    MovePlayerRot, MovePlayerStatusOnly,
};
use mc_protocol::{Decode, Packet};
use tracing::debug;

use crate::WorldTime;
use crate::components::{
    ChunkData, ChunkIndex, ChunkPos, Connection, ConnectionState, EntityId, InPlayState,
    NeedsSpawnChunks, PacketBuffer, Position, ProtocolState, Rotation,
};
use crate::packets::{
    create_chunk_batch_finished, create_game_event_start_waiting, create_keepalive,
    create_play_login, create_player_position, create_set_center_chunk, create_set_time,
    encode_packet,
};

fn send_play_login(buffer: &mut PacketBuffer, entity_id: i32) {
    let result: anyhow::Result<Vec<u8>> = create_play_login(entity_id);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(PlayLogin::ID, &data));
    }
}

fn send_player_position(buffer: &mut PacketBuffer, x: f64, y: f64, z: f64, teleport_id: i32) {
    let result: anyhow::Result<Vec<u8>> = create_player_position(x, y, z, teleport_id);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(PlayerPosition::ID, &data));
    }
}

fn send_game_event_start_waiting(buffer: &mut PacketBuffer) {
    let result: anyhow::Result<Vec<u8>> = create_game_event_start_waiting();
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(GameEvent::ID, &data));
    }
}

fn send_set_center_chunk(buffer: &mut PacketBuffer, x: i32, z: i32) {
    let result: anyhow::Result<Vec<u8>> = create_set_center_chunk(x, z);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(SetChunkCacheCenter::ID, &data));
    }
}

fn send_set_time(buffer: &mut PacketBuffer, world_age: i64, time_of_day: i64) {
    let result: anyhow::Result<Vec<u8>> = create_set_time(world_age, time_of_day);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(SetTime::ID, &data));
    }
}

fn send_keepalive(buffer: &mut PacketBuffer) {
    let result: anyhow::Result<Vec<u8>> = create_keepalive();
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(ClientboundKeepAlive::ID, &data));
    }
}

fn send_chunk_batch_finished(buffer: &mut PacketBuffer, count: i32) {
    let result: anyhow::Result<Vec<u8>> = create_chunk_batch_finished(count);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(ChunkBatchFinished::ID, &data));
    }
}

/// Play module - handles gameplay
#[derive(Component)]
pub struct PlayModule;

impl Module for PlayModule {
    fn module(world: &World) {
        world.module::<PlayModule>("play");

        // Register components
        world.component::<InPlayState>();

        // Send spawn data to new players - use run() to access world for chunk lookup
        world
            .system_named::<(
                &mut PacketBuffer,
                &Position,
                &EntityId,
                &ChunkIndex,
                &WorldTime,
            )>("SendSpawnData")
            .with(NeedsSpawnChunks)
            .with(Connection)
            .run(|mut it| {
                while it.next() {
                    let mut buffer = it.field_mut::<PacketBuffer>(0);
                    let positions = it.field::<Position>(1);
                    let entity_ids = it.field::<EntityId>(2);
                    let chunk_index = &it.field::<ChunkIndex>(3)[0];
                    let world_time = &it.field::<WorldTime>(4)[0];

                    for i in it.iter() {
                        let pos = &positions[i];
                        let entity_id = &entity_ids[i];
                        let buf = &mut buffer[i];

                        // Send Play Login
                        send_play_login(buf, entity_id.0);

                        // Send player position
                        send_player_position(buf, pos.x, pos.y, pos.z, 1);

                        // Send game event (start waiting for chunks)
                        send_game_event_start_waiting(buf);

                        // Set center chunk
                        let (cx, cz) = pos.chunk_pos();
                        send_set_center_chunk(buf, cx, cz);

                        // Send chunks
                        let chunks = collect_chunks_for_player(chunk_index, 8, it.world());
                        send_chunks_to_buffer(buf, &chunks);

                        // Send time
                        send_set_time(buf, world_time.world_age, world_time.time_of_day);

                        // Send initial keepalive
                        send_keepalive(buf);

                        // Remove NeedsSpawnChunks and add InPlayState
                        let entity = it.entity(i);
                        entity.remove(NeedsSpawnChunks);
                        entity.add(InPlayState);

                        tracing::info!("Player entered play state");
                    }
                }
            });

        // Handle play packets
        world
            .system_named::<(
                &ProtocolState,
                &mut PacketBuffer,
                &mut Position,
                &mut Rotation,
            )>("HandlePlayPackets")
            .with(Connection)
            .with(InPlayState)
            .each(|(state, buffer, pos, rot)| {
                if state.0 != ConnectionState::Play {
                    return;
                }

                while let Some((packet_id, data)) = buffer.pop_incoming() {
                    handle_play_packet(packet_id, &data, pos, rot);
                }
            });

        // Periodic keepalive (every ~15 seconds = 300 ticks)
        world
            .system_named::<(&mut PacketBuffer, &WorldTime)>("SendKeepAlive")
            .with(Connection)
            .with(InPlayState)
            .each(|(buffer, world_time)| {
                // Send keepalive every 300 ticks (15 seconds at 20 TPS)
                if world_time.world_age % 300 == 0 {
                    send_keepalive(buffer);
                }
            });
    }
}

fn collect_chunks_for_player(
    chunk_index: &ChunkIndex,
    view_distance: i32,
    world: WorldRef<'_>,
) -> Vec<Bytes> {
    let mut chunks = Vec::new();

    for cx in -view_distance..=view_distance {
        for cz in -view_distance..=view_distance {
            let pos = ChunkPos::new(cx, cz);
            if let Some(chunk_entity) = chunk_index.get(&pos) {
                let entity_view = world.entity_from_id(chunk_entity);
                entity_view.try_get::<&ChunkData>(|chunk_data| {
                    chunks.push(Bytes::clone(&chunk_data.encoded));
                });
            }
        }
    }

    chunks
}

fn send_chunks_to_buffer(buffer: &mut PacketBuffer, chunks: &[Bytes]) {
    // Start chunk batch
    buffer.push_outgoing(encode_packet(ChunkBatchStart::ID, &[]));

    for chunk_data in chunks {
        let packet = encode_packet(LevelChunkWithLight::ID, chunk_data);
        buffer.push_outgoing(packet);
    }

    // Finish chunk batch
    send_chunk_batch_finished(buffer, chunks.len() as i32);
}

fn handle_play_packet(packet_id: i32, data: &[u8], pos: &mut Position, rot: &mut Rotation) {
    let mut cursor = std::io::Cursor::new(data);

    match packet_id {
        AcceptTeleportation::ID => {
            if let Ok(teleport_id) = mc_protocol::read_varint(&mut cursor) {
                debug!("Client accepted teleport: {}", teleport_id);
            }
        }
        ServerboundKeepAlive::ID => {
            if let Ok(ka_id) = i64::decode(&mut cursor) {
                debug!("Keep alive response: {}", ka_id);
            }
        }
        MovePlayerPos::ID => {
            // X, Y, Z, On Ground
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
        MovePlayerPosRot::ID => {
            // X, Y, Z, Yaw, Pitch, On Ground
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
        MovePlayerRot::ID => {
            // Yaw, Pitch, On Ground
            if let (Ok(yaw), Ok(pitch)) = (f32::decode(&mut cursor), f32::decode(&mut cursor)) {
                rot.yaw = yaw;
                rot.pitch = pitch;
            }
        }
        MovePlayerStatusOnly::ID => {
            // Just On Ground flag, ignore
        }
        _ => {
            // Other packets - ignore for now
        }
    }
}
