use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{BufMut, Bytes, BytesMut};
use mc_protocol::{Decode, Encode, write_varint};

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

/// Encode a packet with just the ID and data (no length prefix)
pub fn encode_packet_data(packet_id: i32, data: &[u8]) -> Bytes {
    let mut packet_id_bytes = Vec::new();
    write_varint(&mut packet_id_bytes, packet_id).expect("varint write");

    let mut buf = BytesMut::with_capacity(packet_id_bytes.len() + data.len());
    buf.put_slice(&packet_id_bytes);
    buf.put_slice(data);
    buf.freeze()
}

/// Convert JSON value to network NBT (nameless root compound)
pub fn json_to_network_nbt(json: &serde_json::Value) -> Vec<u8> {
    let nbt_bytes = fastnbt::to_bytes(json).expect("fastnbt serialize");

    if nbt_bytes.len() >= 3 && nbt_bytes[0] == 0x0A {
        let name_len = u16::from_be_bytes([nbt_bytes[1], nbt_bytes[2]]) as usize;
        let skip = 3 + name_len;

        let mut result = Vec::with_capacity(nbt_bytes.len() - name_len - 2);
        result.push(0x0A);
        result.extend_from_slice(&nbt_bytes[skip..]);
        result
    } else {
        nbt_bytes
    }
}

/// Generate offline-mode UUID from username (UUID v3 from "OfflinePlayer:<name>")
#[must_use]
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

/// Parse a handshake packet, returns (protocol_version, next_state)
pub fn parse_handshake(data: &[u8]) -> anyhow::Result<(i32, i32)> {
    let mut cursor = Cursor::new(data);
    let protocol_version = mc_protocol::read_varint(&mut cursor)?;
    let _server_address = String::decode(&mut cursor)?;
    let _server_port = cursor.read_u16::<BigEndian>()?;
    let next_state = mc_protocol::read_varint(&mut cursor)?;
    Ok((protocol_version, next_state))
}

/// Parse a login start packet, returns (name, uuid)
pub fn parse_login_start(data: &[u8]) -> anyhow::Result<(String, u128)> {
    let mut cursor = Cursor::new(data);
    let name = String::decode(&mut cursor)?;
    let uuid = mc_protocol::Uuid::decode(&mut cursor)?;
    Ok((name, uuid.0))
}

/// Create login success response data
pub fn create_login_success(uuid: u128, name: &str) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    mc_protocol::Uuid(uuid).encode(&mut data)?;
    name.to_string().encode(&mut data)?;
    write_varint(&mut data, 0)?; // 0 properties
    Ok(data)
}

/// Create status response JSON
pub fn create_status_response() -> anyhow::Result<Vec<u8>> {
    use serde::Serialize;

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
            text: "A Rust Minecraft Server (Flecs ECS)".to_string(),
        },
        enforces_secure_chat: false,
    };

    let json = serde_json::to_string(&status)?;
    let mut data = Vec::new();
    json.encode(&mut data)?;
    Ok(data)
}

/// Create known packs packet data
pub fn create_known_packs() -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, 1)?; // 1 pack
    "minecraft".to_string().encode(&mut data)?; // namespace
    "core".to_string().encode(&mut data)?; // id
    "1.21".to_string().encode(&mut data)?; // version
    Ok(data)
}

/// Create Play Login packet data
pub fn create_play_login(entity_id: i32) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();

    // Entity ID (Int)
    data.write_i32::<BigEndian>(entity_id)?;

    // Is Hardcore (Boolean)
    false.encode(&mut data)?;

    // Dimension Count (VarInt) + Dimension Names
    write_varint(&mut data, 1)?;
    "minecraft:overworld".to_string().encode(&mut data)?;

    // Max Players (VarInt)
    write_varint(&mut data, 100)?;

    // View Distance (VarInt)
    write_varint(&mut data, 8)?;

    // Simulation Distance (VarInt)
    write_varint(&mut data, 8)?;

    // Reduced Debug Info (Boolean)
    false.encode(&mut data)?;

    // Enable Respawn Screen (Boolean)
    true.encode(&mut data)?;

    // Do Limited Crafting (Boolean)
    false.encode(&mut data)?;

    // Dimension Type (VarInt - registry ID)
    write_varint(&mut data, 0)?;

    // Dimension Name (Identifier)
    "minecraft:overworld".to_string().encode(&mut data)?;

    // Hashed Seed (Long)
    data.write_i64::<BigEndian>(0)?;

    // Game Mode (Unsigned Byte)
    data.write_u8(1)?; // Creative

    // Previous Game Mode (Byte)
    data.write_i8(-1)?;

    // Is Debug (Boolean)
    false.encode(&mut data)?;

    // Is Flat (Boolean)
    true.encode(&mut data)?;

    // Has Death Location (Boolean)
    false.encode(&mut data)?;

    // Portal Cooldown (VarInt)
    write_varint(&mut data, 0)?;

    // Sea Level (VarInt)
    write_varint(&mut data, 63)?;

    // Enforces Secure Chat (Boolean)
    false.encode(&mut data)?;

    Ok(data)
}

/// Create player position packet data
pub fn create_player_position(x: f64, y: f64, z: f64, teleport_id: i32) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();

    // Teleport ID (VarInt)
    write_varint(&mut data, teleport_id)?;

    // X, Y, Z (Double)
    data.write_f64::<BigEndian>(x)?;
    data.write_f64::<BigEndian>(y)?;
    data.write_f64::<BigEndian>(z)?;

    // Velocity X, Y, Z (Double)
    data.write_f64::<BigEndian>(0.0)?;
    data.write_f64::<BigEndian>(0.0)?;
    data.write_f64::<BigEndian>(0.0)?;

    // Yaw, Pitch (Float)
    data.write_f32::<BigEndian>(0.0)?;
    data.write_f32::<BigEndian>(0.0)?;

    // Flags (Int bitfield) - 0 means all absolute
    data.write_i32::<BigEndian>(0)?;

    Ok(data)
}

/// Create game event packet (start waiting for chunks)
pub fn create_game_event_start_waiting() -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    data.write_u8(13)?; // Event: Start waiting for level chunks
    data.write_f32::<BigEndian>(0.0)?;
    Ok(data)
}

/// Create set center chunk packet
pub fn create_set_center_chunk(x: i32, z: i32) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, x)?;
    write_varint(&mut data, z)?;
    Ok(data)
}

/// Create set time packet
pub fn create_set_time(world_age: i64, time_of_day: i64) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    data.write_i64::<BigEndian>(world_age)?;
    data.write_i64::<BigEndian>(time_of_day)?;
    false.encode(&mut data)?; // tick_day_time = false (fixed time)
    Ok(data)
}

/// Create keep-alive packet
pub fn create_keepalive() -> anyhow::Result<Vec<u8>> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time")
        .as_millis() as i64;

    let mut data = Vec::new();
    data.write_i64::<BigEndian>(timestamp)?;
    Ok(data)
}

/// Create chunk batch finished packet
pub fn create_chunk_batch_finished(count: i32) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    write_varint(&mut data, count)?;
    Ok(data)
}
