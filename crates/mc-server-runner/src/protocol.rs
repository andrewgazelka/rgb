//! Protocol helpers - packet encoding and creation

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{BufMut, Bytes, BytesMut};
use mc_protocol::{Decode, Encode, nbt, write_varint};
use serde::Serialize;

use crate::components::PacketBuffer;

// ============================================================================
// Packet encoding
// ============================================================================

/// Encode a packet with ID and data into a length-prefixed packet
pub fn encode_packet(packet_id: i32, data: &[u8]) -> Bytes {
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

// ============================================================================
// Handshake packets
// ============================================================================

/// Parse a handshake packet, returns (protocol_version, next_state)
pub fn parse_handshake(data: &[u8]) -> eyre::Result<(i32, i32)> {
    let mut cursor = std::io::Cursor::new(data);
    let protocol_version = mc_protocol::read_varint(&mut cursor)?;
    let _server_address = String::decode(&mut cursor)?;
    let _server_port = cursor.read_u16::<BigEndian>()?;
    let next_state = mc_protocol::read_varint(&mut cursor)?;
    Ok((protocol_version, next_state))
}

/// Create status response JSON
pub fn create_status_response() -> eyre::Result<Vec<u8>> {
    #[derive(Serialize)]
    struct ServerStatus {
        version: Version,
        players: Players,
        description: Description,
        #[serde(rename = "enforcesSecureChat")]
        enforces_secure_chat: bool,
    }

    #[derive(Serialize)]
    struct Version {
        name: String,
        protocol: i32,
    }

    #[derive(Serialize)]
    struct Players {
        max: i32,
        online: i32,
        sample: Vec<PlayerSample>,
    }

    #[derive(Serialize)]
    struct PlayerSample {
        name: String,
        id: String,
    }

    #[derive(Serialize)]
    struct Description {
        text: String,
    }

    let status = ServerStatus {
        version: Version {
            name: mc_data::PROTOCOL_NAME.to_string(),
            protocol: mc_data::PROTOCOL_VERSION,
        },
        players: Players {
            max: 100,
            online: 0,
            sample: vec![],
        },
        description: Description {
            text: "A Rust Minecraft Server (RGB ECS)".to_string(),
        },
        enforces_secure_chat: false,
    };

    let json = serde_json::to_string(&status)?;
    let mut data = Vec::new();
    json.encode(&mut data)?;
    Ok(data)
}

// ============================================================================
// Login packets
// ============================================================================

pub fn offline_uuid(name: &str) -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let input = format!("OfflinePlayer:{}", name);
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash1 = hasher.finish();
    input.hash(&mut hasher);
    let hash2 = hasher.finish();

    let mut uuid = ((hash1 as u128) << 64) | (hash2 as u128);
    uuid = (uuid & 0xFFFFFFFFFFFF0FFFFFFFFFFFFFFF) | 0x00000000000030000000000000000000;
    uuid = (uuid & 0xFFFFFFFFFFFFFFFF3FFFFFFFFFFFFFFF) | 0x00000000000000008000000000000000;
    uuid
}

pub fn parse_login_start(data: &[u8]) -> eyre::Result<(String, u128)> {
    let mut cursor = std::io::Cursor::new(data);
    let name = String::decode(&mut cursor)?;
    let uuid = mc_protocol::Uuid::decode(&mut cursor)?;
    Ok((name, uuid.0))
}

pub fn create_login_success(uuid: u128, name: &str) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    mc_protocol::Uuid(uuid).encode(&mut data)?;
    name.to_string().encode(&mut data)?;
    write_varint(&mut data, 0)?; // 0 properties
    Ok(data)
}

pub fn create_known_packs() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, 1)?;
    "minecraft".to_string().encode(&mut data)?;
    "core".to_string().encode(&mut data)?;
    "1.21".to_string().encode(&mut data)?;
    Ok(data)
}

// ============================================================================
// Play packets
// ============================================================================

pub fn create_play_login(entity_id: i32) -> eyre::Result<Vec<u8>> {
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

pub fn create_player_position(x: f64, y: f64, z: f64, teleport_id: i32) -> eyre::Result<Vec<u8>> {
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

pub fn create_game_event_start_waiting() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    data.write_u8(13)?;
    data.write_f32::<BigEndian>(0.0)?;
    Ok(data)
}

pub fn create_set_center_chunk(x: i32, z: i32) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, x)?;
    write_varint(&mut data, z)?;
    Ok(data)
}

pub fn create_set_time(world_age: i64, time_of_day: i64) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    data.write_i64::<BigEndian>(world_age)?;
    data.write_i64::<BigEndian>(time_of_day)?;
    false.encode(&mut data)?;
    Ok(data)
}

pub fn create_keepalive() -> eyre::Result<Vec<u8>> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time")
        .as_millis() as i64;
    let mut data = Vec::new();
    data.write_i64::<BigEndian>(timestamp)?;
    Ok(data)
}

pub fn create_chunk_batch_finished(count: i32) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, count)?;
    Ok(data)
}

pub fn create_action_bar_text(text: &str) -> eyre::Result<Vec<u8>> {
    let compound = nbt! {
        "text" => text,
    };
    Ok(compound.to_network_bytes())
}

// ============================================================================
// Packet IDs (from mc_data)
// ============================================================================

pub mod packet_ids {
    use mc_data::play::clientbound::{
        ChunkBatchFinished, ChunkBatchStart, GameEvent, KeepAlive as ClientboundKeepAlive,
        LevelChunkWithLight, Login as PlayLogin, PlayerPosition, SetActionBarText,
        SetChunkCacheCenter, SetTime,
    };
    use mc_protocol::Packet;

    pub const PLAY_LOGIN: i32 = PlayLogin::ID;
    pub const GAME_EVENT: i32 = GameEvent::ID;
    pub const SET_CHUNK_CENTER: i32 = SetChunkCacheCenter::ID;
    pub const SET_TIME: i32 = SetTime::ID;
    pub const PLAYER_POSITION: i32 = PlayerPosition::ID;
    pub const KEEPALIVE: i32 = ClientboundKeepAlive::ID;
    pub const CHUNK_BATCH_START: i32 = ChunkBatchStart::ID;
    pub const CHUNK_BATCH_FINISHED: i32 = ChunkBatchFinished::ID;
    pub const LEVEL_CHUNK: i32 = LevelChunkWithLight::ID;
    pub const ACTION_BAR: i32 = SetActionBarText::ID;
}

// ============================================================================
// Buffer helpers
// ============================================================================

pub fn send_status_response(buffer: &mut PacketBuffer) {
    if let Ok(response_data) = create_status_response() {
        let packet = encode_packet(0, &response_data);
        buffer.push_outgoing(packet);
    }
}

pub fn send_login_success(buffer: &mut PacketBuffer, uuid: u128, name: &str) {
    if let Ok(response_data) = create_login_success(uuid, name) {
        let packet = encode_packet(2, &response_data);
        buffer.push_outgoing(packet);
    }
}

pub fn send_known_packs(buffer: &mut PacketBuffer) {
    if let Ok(data) = create_known_packs() {
        let packet = encode_packet(14, &data);
        buffer.push_outgoing(packet);
    }
}

pub fn send_play_login(buffer: &mut PacketBuffer, entity_id: i32) {
    if let Ok(data) = create_play_login(entity_id) {
        buffer.push_outgoing(encode_packet(packet_ids::PLAY_LOGIN, &data));
    }
}

pub fn send_player_position(buffer: &mut PacketBuffer, x: f64, y: f64, z: f64, teleport_id: i32) {
    if let Ok(data) = create_player_position(x, y, z, teleport_id) {
        buffer.push_outgoing(encode_packet(packet_ids::PLAYER_POSITION, &data));
    }
}

pub fn send_game_event_start_waiting(buffer: &mut PacketBuffer) {
    if let Ok(data) = create_game_event_start_waiting() {
        buffer.push_outgoing(encode_packet(packet_ids::GAME_EVENT, &data));
    }
}

pub fn send_set_center_chunk(buffer: &mut PacketBuffer, x: i32, z: i32) {
    if let Ok(data) = create_set_center_chunk(x, z) {
        buffer.push_outgoing(encode_packet(packet_ids::SET_CHUNK_CENTER, &data));
    }
}

pub fn send_set_time(buffer: &mut PacketBuffer, world_age: i64, time_of_day: i64) {
    if let Ok(data) = create_set_time(world_age, time_of_day) {
        buffer.push_outgoing(encode_packet(packet_ids::SET_TIME, &data));
    }
}

pub fn send_keepalive(buffer: &mut PacketBuffer) {
    if let Ok(data) = create_keepalive() {
        buffer.push_outgoing(encode_packet(packet_ids::KEEPALIVE, &data));
    }
}

pub fn send_chunk_batch_finished(buffer: &mut PacketBuffer, count: i32) {
    if let Ok(data) = create_chunk_batch_finished(count) {
        buffer.push_outgoing(encode_packet(packet_ids::CHUNK_BATCH_FINISHED, &data));
    }
}

pub fn send_action_bar(buffer: &mut PacketBuffer, text: &str) {
    if let Ok(data) = create_action_bar_text(text) {
        buffer.push_outgoing(encode_packet(packet_ids::ACTION_BAR, &data));
    }
}

pub fn send_chunks_to_buffer(buffer: &mut PacketBuffer, chunks: &[Bytes]) {
    buffer.push_outgoing(encode_packet(packet_ids::CHUNK_BATCH_START, &[]));

    for chunk_data in chunks {
        let packet = encode_packet(packet_ids::LEVEL_CHUNK, chunk_data);
        buffer.push_outgoing(packet);
    }

    send_chunk_batch_finished(buffer, chunks.len() as i32);
}
