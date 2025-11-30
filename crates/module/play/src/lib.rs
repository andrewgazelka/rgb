//! Play module - handles play state

use bytes::Bytes;
use flecs_ecs::prelude::*;
use mc_data::play::clientbound::{
    ChunkBatchFinished, ChunkBatchStart, GameEvent, KeepAlive as ClientboundKeepAlive,
    LevelChunkWithLight, Login as PlayLogin, PlayerPosition, SetActionBarText, SetChunkCacheCenter,
    SetTime,
};
use mc_data::play::serverbound::{
    AcceptTeleportation, KeepAlive as ServerboundKeepAlive, MovePlayerPos, MovePlayerPosRot,
    MovePlayerRot, MovePlayerStatusOnly,
};
use mc_protocol::{Decode, Packet};
use mc_server_lib::{
    ChunkData, ChunkIndex, ChunkPos, Connection, ConnectionState, EntityId, InPlayState,
    NeedsSpawnChunks, PacketBuffer, PacketHandlerRegistration, Position, Rotation, TpsTracker,
    WorldTime, create_action_bar_text, create_chunk_batch_finished,
    create_game_event_start_waiting, create_keepalive, create_play_login, create_player_position,
    create_set_center_chunk, create_set_time, encode_packet,
};
use module_loader::register_plugin;
use tracing::debug;

fn send_play_login(buffer: &mut PacketBuffer, entity_id: i32) {
    let result: eyre::Result<Vec<u8>> = create_play_login(entity_id);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(PlayLogin::ID, &data));
    }
}

fn send_player_position(buffer: &mut PacketBuffer, x: f64, y: f64, z: f64, teleport_id: i32) {
    let result: eyre::Result<Vec<u8>> = create_player_position(x, y, z, teleport_id);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(PlayerPosition::ID, &data));
    }
}

fn send_game_event_start_waiting(buffer: &mut PacketBuffer) {
    let result: eyre::Result<Vec<u8>> = create_game_event_start_waiting();
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(GameEvent::ID, &data));
    }
}

fn send_set_center_chunk(buffer: &mut PacketBuffer, x: i32, z: i32) {
    let result: eyre::Result<Vec<u8>> = create_set_center_chunk(x, z);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(SetChunkCacheCenter::ID, &data));
    }
}

fn send_set_time(buffer: &mut PacketBuffer, world_age: i64, time_of_day: i64) {
    let result: eyre::Result<Vec<u8>> = create_set_time(world_age, time_of_day);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(SetTime::ID, &data));
    }
}

fn send_keepalive(buffer: &mut PacketBuffer) {
    let result: eyre::Result<Vec<u8>> = create_keepalive();
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(ClientboundKeepAlive::ID, &data));
    }
}

fn send_chunk_batch_finished(buffer: &mut PacketBuffer, count: i32) {
    let result: eyre::Result<Vec<u8>> = create_chunk_batch_finished(count);
    if let Ok(data) = result {
        buffer.push_outgoing(encode_packet(ChunkBatchFinished::ID, &data));
    }
}

fn send_action_bar(buffer: &mut PacketBuffer, text: &str) {
    if let Ok(data) = create_action_bar_text(text) {
        buffer.push_outgoing(encode_packet(SetActionBarText::ID, &data));
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

                        // 1. Send Play Login - creates player and level on client
                        send_play_login(buf, entity_id.value);

                        // 2. Send game event (start waiting for chunks) - client waits for chunks
                        send_game_event_start_waiting(buf);

                        // 3. Set center chunk
                        let (cx, cz) = pos.chunk_pos();
                        send_set_center_chunk(buf, cx, cz);

                        // 4. Send chunks
                        let chunks = collect_chunks_for_player(chunk_index, 8, it.world());
                        send_chunks_to_buffer(buf, &chunks);

                        // 5. Send time
                        send_set_time(buf, world_time.world_age, world_time.time_of_day);

                        // 6. Send player position AFTER chunks are loaded
                        send_player_position(buf, pos.x, pos.y, pos.z, 1);

                        // 7. Send initial keepalive
                        send_keepalive(buf);

                        // Remove NeedsSpawnChunks and add InPlayState
                        let entity = it.entity(i);
                        entity.remove(NeedsSpawnChunks);
                        entity.add(InPlayState);

                        tracing::info!("Player entered play state");
                    }
                }
            });

        // Register packet handlers using the new dispatch system
        world.register_handler(
            "AcceptTeleportation",
            ConnectionState::Play,
            AcceptTeleportation::ID,
            0,
            handle_accept_teleportation,
        );
        world.register_handler(
            "KeepAlive",
            ConnectionState::Play,
            ServerboundKeepAlive::ID,
            0,
            handle_keep_alive,
        );
        world.register_handler(
            "MovePlayerPos",
            ConnectionState::Play,
            MovePlayerPos::ID,
            0,
            handle_move_player_pos,
        );
        world.register_handler(
            "MovePlayerPosRot",
            ConnectionState::Play,
            MovePlayerPosRot::ID,
            0,
            handle_move_player_pos_rot,
        );
        world.register_handler(
            "MovePlayerRot",
            ConnectionState::Play,
            MovePlayerRot::ID,
            0,
            handle_move_player_rot,
        );
        world.register_handler(
            "MovePlayerStatusOnly",
            ConnectionState::Play,
            MovePlayerStatusOnly::ID,
            0,
            handle_move_player_status_only,
        );

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

        // Send position and TPS to action bar (every 10 ticks = 0.5 seconds)
        world
            .system_named::<(&mut PacketBuffer, &Position, &WorldTime, &TpsTracker)>(
                "SendPositionActionBar",
            )
            .with(Connection)
            .with(InPlayState)
            .each(|(buffer, pos, world_time, tps)| {
                // Update every 10 ticks (0.5 seconds at 20 TPS)
                if world_time.world_age % 10 == 0 {
                    let text = format!(
                        "X: {:.1} Y: {:.1} Z: {:.1} | TPS: {:.1}:{:.1}:{:.1}",
                        pos.x, pos.y, pos.z, tps.tps_5s, tps.tps_15s, tps.tps_1m
                    );
                    send_action_bar(buffer, &text);
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

// Individual packet handlers for the new dispatch system

fn handle_accept_teleportation(_entity: EntityView<'_>, data: &[u8]) {
    let mut cursor = std::io::Cursor::new(data);
    if let Ok(teleport_id) = mc_protocol::read_varint(&mut cursor) {
        debug!("Client accepted teleport: {}", teleport_id);
    }
}

fn handle_keep_alive(_entity: EntityView<'_>, data: &[u8]) {
    let mut cursor = std::io::Cursor::new(data);
    if let Ok(ka_id) = i64::decode(&mut cursor) {
        debug!("Keep alive response: {}", ka_id);
    }
}

fn handle_move_player_pos(entity: EntityView<'_>, data: &[u8]) {
    let mut cursor = std::io::Cursor::new(data);
    if let (Ok(x), Ok(y), Ok(z)) = (
        f64::decode(&mut cursor),
        f64::decode(&mut cursor),
        f64::decode(&mut cursor),
    ) {
        debug!("MovePlayerPos: x={}, y={}, z={}", x, y, z);
        entity.get::<&mut Position>(|pos| {
            pos.x = x;
            pos.y = y;
            pos.z = z;
        });
    }
}

fn handle_move_player_pos_rot(entity: EntityView<'_>, data: &[u8]) {
    let mut cursor = std::io::Cursor::new(data);
    if let (Ok(x), Ok(y), Ok(z), Ok(yaw), Ok(pitch)) = (
        f64::decode(&mut cursor),
        f64::decode(&mut cursor),
        f64::decode(&mut cursor),
        f32::decode(&mut cursor),
        f32::decode(&mut cursor),
    ) {
        debug!("MovePlayerPosRot: x={}, y={}, z={}", x, y, z);
        entity.get::<(&mut Position, &mut Rotation)>(|(pos, rot)| {
            pos.x = x;
            pos.y = y;
            pos.z = z;
            rot.yaw = yaw;
            rot.pitch = pitch;
        });
    }
}

fn handle_move_player_rot(entity: EntityView<'_>, data: &[u8]) {
    let mut cursor = std::io::Cursor::new(data);
    if let (Ok(yaw), Ok(pitch)) = (f32::decode(&mut cursor), f32::decode(&mut cursor)) {
        entity.get::<&mut Rotation>(|rot| {
            rot.yaw = yaw;
            rot.pitch = pitch;
        });
    }
}

fn handle_move_player_status_only(_entity: EntityView<'_>, _data: &[u8]) {
    // Just On Ground flag, ignore
}

register_plugin! {
    name: "play",
    version: 1,
    module: PlayModule,
    path: "::play",
}
