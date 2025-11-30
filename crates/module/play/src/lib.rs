//! Play module - handles play state

use byteorder::{BigEndian, WriteBytesExt};
use bytes::{BufMut, Bytes, BytesMut};
use flecs_ecs::prelude::*;
use mc_data::play::clientbound::{
    ChunkBatchFinished, ChunkBatchStart, GameEvent, KeepAlive as ClientboundKeepAlive,
    LevelChunkWithLight, Login as PlayLogin, PlayerPosition, SetActionBarText, SetChunkCacheCenter,
    SetTime,
};
use mc_protocol::{nbt, write_varint, Decode, Encode, Packet};
use module_chunk::{ChunkData, ChunkIndex, ChunkPos};
use module_loader::register_module;
use module_login::{EntityId, InPlayState, NeedsSpawnChunks, Position, Rotation};
use module_network_components::{Connection, PacketBuffer};
use module_time_components::{TpsTracker, WorldTime};
use tracing::debug;

// ============================================================================
// Packet helpers
// ============================================================================

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

fn create_play_login(entity_id: i32) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();

    data.write_i32::<BigEndian>(entity_id)?;
    false.encode(&mut data)?; // is_hardcore
    write_varint(&mut data, 1)?; // 1 dimension
    "minecraft:overworld".to_string().encode(&mut data)?;
    write_varint(&mut data, 100)?; // max_players
    write_varint(&mut data, 8)?; // view_distance
    write_varint(&mut data, 8)?; // simulation_distance
    false.encode(&mut data)?; // reduced_debug_info
    true.encode(&mut data)?; // enable_respawn_screen
    false.encode(&mut data)?; // do_limited_crafting
    write_varint(&mut data, 0)?; // dimension_type (registry ID)
    "minecraft:overworld".to_string().encode(&mut data)?; // dimension
    data.write_i64::<BigEndian>(0)?; // hashed_seed
    data.write_u8(1)?; // game_mode (creative)
    data.write_i8(-1)?; // previous_game_mode
    false.encode(&mut data)?; // is_debug
    true.encode(&mut data)?; // is_flat
    false.encode(&mut data)?; // has_death_location
    write_varint(&mut data, 0)?; // portal_cooldown
    write_varint(&mut data, 63)?; // sea_level
    false.encode(&mut data)?; // enforces_secure_chat

    Ok(data)
}

fn create_player_position(x: f64, y: f64, z: f64, teleport_id: i32) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, teleport_id)?;
    data.write_f64::<BigEndian>(x)?;
    data.write_f64::<BigEndian>(y)?;
    data.write_f64::<BigEndian>(z)?;
    data.write_f64::<BigEndian>(0.0)?; // vel_x
    data.write_f64::<BigEndian>(0.0)?; // vel_y
    data.write_f64::<BigEndian>(0.0)?; // vel_z
    data.write_f32::<BigEndian>(0.0)?; // yaw
    data.write_f32::<BigEndian>(0.0)?; // pitch
    data.write_i32::<BigEndian>(0)?; // flags
    Ok(data)
}

fn create_game_event_start_waiting() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    data.write_u8(13)?;
    data.write_f32::<BigEndian>(0.0)?;
    Ok(data)
}

fn create_set_center_chunk(x: i32, z: i32) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, x)?;
    write_varint(&mut data, z)?;
    Ok(data)
}

fn create_set_time(world_age: i64, time_of_day: i64) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    data.write_i64::<BigEndian>(world_age)?;
    data.write_i64::<BigEndian>(time_of_day)?;
    false.encode(&mut data)?;
    Ok(data)
}

fn create_keepalive() -> eyre::Result<Vec<u8>> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time")
        .as_millis() as i64;
    let mut data = Vec::new();
    data.write_i64::<BigEndian>(timestamp)?;
    Ok(data)
}

fn create_chunk_batch_finished(count: i32) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, count)?;
    Ok(data)
}

fn create_action_bar_text(text: &str) -> eyre::Result<Vec<u8>> {
    let compound = nbt! {
        "text" => text,
    };
    Ok(compound.to_network_bytes())
}

fn send_play_login(buffer: &mut PacketBuffer, entity_id: i32) {
    if let Ok(data) = create_play_login(entity_id) {
        buffer.push_outgoing(encode_packet(PlayLogin::ID, &data));
    }
}

fn send_player_position(buffer: &mut PacketBuffer, x: f64, y: f64, z: f64, teleport_id: i32) {
    if let Ok(data) = create_player_position(x, y, z, teleport_id) {
        buffer.push_outgoing(encode_packet(PlayerPosition::ID, &data));
    }
}

fn send_game_event_start_waiting(buffer: &mut PacketBuffer) {
    if let Ok(data) = create_game_event_start_waiting() {
        buffer.push_outgoing(encode_packet(GameEvent::ID, &data));
    }
}

fn send_set_center_chunk(buffer: &mut PacketBuffer, x: i32, z: i32) {
    if let Ok(data) = create_set_center_chunk(x, z) {
        buffer.push_outgoing(encode_packet(SetChunkCacheCenter::ID, &data));
    }
}

fn send_set_time(buffer: &mut PacketBuffer, world_age: i64, time_of_day: i64) {
    if let Ok(data) = create_set_time(world_age, time_of_day) {
        buffer.push_outgoing(encode_packet(SetTime::ID, &data));
    }
}

fn send_keepalive(buffer: &mut PacketBuffer) {
    if let Ok(data) = create_keepalive() {
        buffer.push_outgoing(encode_packet(ClientboundKeepAlive::ID, &data));
    }
}

fn send_chunk_batch_finished(buffer: &mut PacketBuffer, count: i32) {
    if let Ok(data) = create_chunk_batch_finished(count) {
        buffer.push_outgoing(encode_packet(ChunkBatchFinished::ID, &data));
    }
}

fn send_action_bar(buffer: &mut PacketBuffer, text: &str) {
    if let Ok(data) = create_action_bar_text(text) {
        buffer.push_outgoing(encode_packet(SetActionBarText::ID, &data));
    }
}

// ============================================================================
// Module
// ============================================================================

/// Play module - handles gameplay
#[derive(Component)]
pub struct PlayModule;

impl Module for PlayModule {
    fn module(world: &World) {
        world.module::<PlayModule>("play");

        // Send spawn data to new players
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

                        send_play_login(buf, entity_id.value);
                        send_game_event_start_waiting(buf);

                        let (cx, cz) = pos.chunk_pos();
                        send_set_center_chunk(buf, cx, cz);

                        let chunks = collect_chunks_for_player(chunk_index, 8, it.world());
                        send_chunks_to_buffer(buf, &chunks);

                        send_set_time(buf, world_time.world_age, world_time.time_of_day);
                        send_player_position(buf, pos.x, pos.y, pos.z, 1);
                        send_keepalive(buf);

                        let entity = it.entity(i);
                        entity.remove(NeedsSpawnChunks);
                        entity.add(InPlayState);

                        tracing::info!("Player entered play state");
                    }
                }
            });

        // Periodic keepalive
        world
            .system_named::<(&mut PacketBuffer, &WorldTime)>("SendKeepAlive")
            .with(Connection)
            .with(InPlayState)
            .each(|(buffer, world_time)| {
                if world_time.world_age % 300 == 0 {
                    send_keepalive(buffer);
                }
            });

        // Send position and TPS to action bar
        world
            .system_named::<(&mut PacketBuffer, &Position, &WorldTime, &TpsTracker)>(
                "SendPositionActionBar",
            )
            .with(Connection)
            .with(InPlayState)
            .each(|(buffer, pos, world_time, tps)| {
                if world_time.world_age % 10 == 0 {
                    let text = format!(
                        "X: {:.1} Y: {:.1} Z: {:.1} | TPS: {:.1}:{:.1}:{:.1}",
                        pos.x, pos.y, pos.z, tps.tps_5s, tps.tps_15s, tps.tps_1m
                    );
                    send_action_bar(buffer, &text);
                }
            });

        // Handle player movement packets directly (without packet dispatch)
        world
            .system_named::<(&mut PacketBuffer, &mut Position, &mut Rotation)>("HandleMovement")
            .with(Connection)
            .with(InPlayState)
            .each(|(buffer, pos, rot)| {
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
                            if let (Ok(yaw), Ok(pitch)) =
                                (f32::decode(&mut cursor), f32::decode(&mut cursor))
                            {
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
    buffer.push_outgoing(encode_packet(ChunkBatchStart::ID, &[]));

    for chunk_data in chunks {
        let packet = encode_packet(LevelChunkWithLight::ID, chunk_data);
        buffer.push_outgoing(packet);
    }

    send_chunk_batch_finished(buffer, chunks.len() as i32);
}

register_module! {
    name: "play",
    version: 1,
    module: PlayModule,
    path: "::play",
}
