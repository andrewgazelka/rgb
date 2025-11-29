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

use crate::components::{
    ChunkData, ChunkIndex, ChunkPos, Connection, ConnectionState, EntityId, InPlayState,
    NeedsSpawnChunks, PacketBuffer, Position, ProtocolState, Rotation,
};
use crate::packets::{
    create_chunk_batch_finished, create_game_event_start_waiting, create_keepalive,
    create_play_login, create_player_position, create_set_center_chunk, create_set_time,
    encode_packet,
};
use crate::WorldTime;

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
                    let buffer = it.field_mut::<PacketBuffer>(0);
                    let positions = it.field::<Position>(1);
                    let entity_ids = it.field::<EntityId>(2);
                    let chunk_index = &it.field::<ChunkIndex>(3)[0];
                    let world_time = &it.field::<WorldTime>(4)[0];

                    for i in it.iter() {
                        let pos = &positions[i];
                        let entity_id = &entity_ids[i];
                        let buf = &mut buffer[i];

                        // Send Play Login
                        if let Some(data) = create_play_login(entity_id.0).ok() {
                            buf.push_outgoing(encode_packet(PlayLogin::ID, &data));
                        }

                        // Send player position
                        if let Some(data) = create_player_position(pos.x, pos.y, pos.z, 1).ok() {
                            buf.push_outgoing(encode_packet(PlayerPosition::ID, &data));
                        }

                        // Send game event (start waiting for chunks)
                        if let Some(data) = create_game_event_start_waiting().ok() {
                            buf.push_outgoing(encode_packet(GameEvent::ID, &data));
                        }

                        // Set center chunk
                        let (cx, cz) = pos.chunk_pos();
                        if let Some(data) = create_set_center_chunk(cx, cz).ok() {
                            buf.push_outgoing(encode_packet(SetChunkCacheCenter::ID, &data));
                        }

                        // Send chunks
                        let chunks = collect_chunks_for_player(chunk_index, 8, it.world());
                        send_chunks_to_buffer(buf, &chunks);

                        // Send time
                        if let Some(data) = create_set_time(world_time.world_age, world_time.time_of_day).ok() {
                            buf.push_outgoing(encode_packet(SetTime::ID, &data));
                        }

                        // Send initial keepalive
                        if let Some(data) = create_keepalive().ok() {
                            buf.push_outgoing(encode_packet(ClientboundKeepAlive::ID, &data));
                        }

                        // Remove NeedsSpawnChunks and add InPlayState
                        let entity = it.entity(i);
                        entity.remove::<NeedsSpawnChunks>();
                        entity.add(InPlayState);

                        tracing::info!("Player entered play state");
                    }
                }
            });

        // Handle play packets
        world
            .system_named::<(&ProtocolState, &mut PacketBuffer, &mut Position, &mut Rotation)>(
                "HandlePlayPackets",
            )
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
                    if let Some(data) = create_keepalive().ok() {
                        buffer.push_outgoing(encode_packet(ClientboundKeepAlive::ID, &data));
                    }
                }
            });
    }
}

fn collect_chunks_for_player(chunk_index: &ChunkIndex, view_distance: i32, world: WorldRef<'_>) -> Vec<Bytes> {
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
    if let Some(data) = create_chunk_batch_finished(chunks.len() as i32).ok() {
        buffer.push_outgoing(encode_packet(ChunkBatchFinished::ID, &data));
    }
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
