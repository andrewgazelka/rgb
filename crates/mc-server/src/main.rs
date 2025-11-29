use std::io::Cursor;
use std::sync::atomic::{AtomicI64, Ordering};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use mc_packets::Packet;
use mc_protocol::{Decode, Encode, Uuid, read_varint, write_varint};
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

// Import packet types for their IDs
use mc_packets::play::clientbound::{
    GameEvent, KeepAlive as ClientboundKeepAlive, LevelChunkWithLight, Login as PlayLogin,
    PlayerPosition, SetChunkCacheCenter,
};
use mc_packets::play::serverbound::{
    AcceptTeleportation, KeepAlive as ServerboundKeepAlive, MovePlayerPos, MovePlayerPosRot,
    MovePlayerRot, MovePlayerStatusOnly,
};

struct DamageType {
    name: &'static str,
    message_id: &'static str,
    exhaustion: f32,
    scaling: &'static str,
    effects: &'static str,
}

const DAMAGE_TYPES: &[DamageType] = &[
    DamageType {
        name: "in_fire",
        message_id: "inFire",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "burning",
    },
    DamageType {
        name: "campfire",
        message_id: "inFire",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "burning",
    },
    DamageType {
        name: "lightning_bolt",
        message_id: "lightningBolt",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "on_fire",
        message_id: "onFire",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "burning",
    },
    DamageType {
        name: "lava",
        message_id: "lava",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "burning",
    },
    DamageType {
        name: "hot_floor",
        message_id: "hotFloor",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "burning",
    },
    DamageType {
        name: "in_wall",
        message_id: "inWall",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "cramming",
        message_id: "cramming",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "drown",
        message_id: "drown",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "drowning",
    },
    DamageType {
        name: "starve",
        message_id: "starve",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "cactus",
        message_id: "cactus",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "fall",
        message_id: "fall",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "ender_pearl",
        message_id: "fall",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "fly_into_wall",
        message_id: "flyIntoWall",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "out_of_world",
        message_id: "outOfWorld",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "generic",
        message_id: "generic",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "magic",
        message_id: "magic",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "wither",
        message_id: "wither",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "dragon_breath",
        message_id: "dragonBreath",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "dry_out",
        message_id: "dryout",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "sweet_berry_bush",
        message_id: "sweetBerryBush",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "poking",
    },
    DamageType {
        name: "freeze",
        message_id: "freeze",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "freezing",
    },
    DamageType {
        name: "stalagmite",
        message_id: "stalagmite",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "falling_block",
        message_id: "fallingBlock",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "falling_anvil",
        message_id: "anvil",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "falling_stalactite",
        message_id: "fallingStalactite",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "sting",
        message_id: "sting",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "mob_attack",
        message_id: "mob",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "mob_attack_no_aggro",
        message_id: "mob",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "player_attack",
        message_id: "player",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "spear",
        message_id: "spear",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "arrow",
        message_id: "arrow",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "trident",
        message_id: "trident",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "mob_projectile",
        message_id: "mob",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "spit",
        message_id: "mob",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "wind_charge",
        message_id: "mob",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "fireworks",
        message_id: "fireworks",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "fireball",
        message_id: "fireball",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "burning",
    },
    DamageType {
        name: "unattributed_fireball",
        message_id: "onFire",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "burning",
    },
    DamageType {
        name: "wither_skull",
        message_id: "witherSkull",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "thrown",
        message_id: "thrown",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "indirect_magic",
        message_id: "indirectMagic",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "thorns",
        message_id: "thorns",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "thorns",
    },
    DamageType {
        name: "explosion",
        message_id: "explosion",
        exhaustion: 0.1,
        scaling: "always",
        effects: "hurt",
    },
    DamageType {
        name: "player_explosion",
        message_id: "explosion.player",
        exhaustion: 0.1,
        scaling: "always",
        effects: "hurt",
    },
    DamageType {
        name: "sonic_boom",
        message_id: "sonic_boom",
        exhaustion: 0.0,
        scaling: "always",
        effects: "hurt",
    },
    DamageType {
        name: "bad_respawn_point",
        message_id: "badRespawnPoint",
        exhaustion: 0.1,
        scaling: "always",
        effects: "hurt",
    },
    DamageType {
        name: "outside_border",
        message_id: "outsideBorder",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "generic_kill",
        message_id: "genericKill",
        exhaustion: 0.0,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
    DamageType {
        name: "mace_smash",
        message_id: "mace_smash",
        exhaustion: 0.1,
        scaling: "when_caused_by_living_non_player",
        effects: "hurt",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Debug, Serialize)]
struct ServerStatus {
    version: Version,
    players: Players,
    description: Description,
    #[serde(rename = "enforcesSecureChat")]
    enforces_secure_chat: bool,
}

#[derive(Debug, Serialize)]
struct Version {
    name: String,
    protocol: i32,
}

#[derive(Debug, Serialize)]
struct Players {
    max: i32,
    online: i32,
    sample: Vec<PlayerSample>,
}

#[derive(Debug, Serialize)]
struct PlayerSample {
    name: String,
    id: String,
}

#[derive(Debug, Serialize)]
struct Description {
    text: String,
}

/// Generate offline-mode UUID from username (UUID v3 from "OfflinePlayer:<name>")
fn offline_uuid(name: &str) -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Simple hash-based UUID for offline mode
    // Real implementation should use UUID v3 with MD5
    let input = format!("OfflinePlayer:{}", name);
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash1 = hasher.finish();
    input.hash(&mut hasher);
    let hash2 = hasher.finish();

    let mut uuid = ((hash1 as u128) << 64) | (hash2 as u128);
    // Set version to 3 and variant
    uuid = (uuid & 0xFFFFFFFFFFFF0FFFFFFFFFFFFFFF) | 0x00000000000030000000000000000000;
    uuid = (uuid & 0xFFFFFFFFFFFFFFFF3FFFFFFFFFFFFFFF) | 0x00000000000000008000000000000000;
    uuid
}

struct Connection {
    stream: TcpStream,
    state: ConnectionState,
    player_name: Option<String>,
    player_uuid: Option<u128>,
    entity_id: i32,
}

impl Connection {
    fn new(stream: TcpStream, entity_id: i32) -> Self {
        Self {
            stream,
            state: ConnectionState::Handshaking,
            player_name: None,
            player_uuid: None,
            entity_id,
        }
    }

    async fn read_packet(&mut self) -> anyhow::Result<(i32, Vec<u8>)> {
        let length = self.read_varint().await?;
        if length == 0 {
            return Ok((-1, vec![]));
        }

        let mut data = vec![0u8; length as usize];
        self.stream.read_exact(&mut data).await?;

        let mut cursor = Cursor::new(&data);
        let packet_id = read_varint(&mut cursor)?;
        let remaining = data[cursor.position() as usize..].to_vec();

        Ok((packet_id, remaining))
    }

    async fn read_varint(&mut self) -> anyhow::Result<i32> {
        let mut result = 0i32;
        let mut shift = 0;
        loop {
            let mut buf = [0u8; 1];
            self.stream.read_exact(&mut buf).await?;
            let byte = buf[0];
            result |= ((byte & 0x7F) as i32) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift >= 32 {
                anyhow::bail!("VarInt too large");
            }
        }
        Ok(result)
    }

    async fn send_packet(&mut self, packet_id: i32, data: &[u8]) -> anyhow::Result<()> {
        let mut packet_id_bytes = Vec::new();
        write_varint(&mut packet_id_bytes, packet_id)?;

        let length = packet_id_bytes.len() + data.len();
        let mut length_bytes = Vec::new();
        write_varint(&mut length_bytes, length as i32)?;

        self.stream.write_all(&length_bytes).await?;
        self.stream.write_all(&packet_id_bytes).await?;
        self.stream.write_all(data).await?;
        self.stream.flush().await?;

        Ok(())
    }

    async fn handle(&mut self) -> anyhow::Result<()> {
        loop {
            let (packet_id, data) = self.read_packet().await?;
            if packet_id == -1 {
                continue;
            }

            let mut cursor = Cursor::new(&data);

            match self.state {
                ConnectionState::Handshaking => {
                    self.handle_handshaking(packet_id, &mut cursor).await?;
                }
                ConnectionState::Status => {
                    if !self.handle_status(packet_id, &mut cursor).await? {
                        break;
                    }
                }
                ConnectionState::Login => {
                    self.handle_login(packet_id, &mut cursor).await?;
                }
                ConnectionState::Configuration => {
                    self.handle_configuration(packet_id, &mut cursor).await?;
                }
                ConnectionState::Play => {
                    self.handle_play(packet_id, &mut cursor).await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_handshaking(
        &mut self,
        packet_id: i32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> anyhow::Result<()> {
        if packet_id == 0 {
            let protocol_version = read_varint(cursor)?;
            let _server_address = String::decode(cursor)?;
            let _server_port = ReadBytesExt::read_u16::<BigEndian>(cursor)?;
            let next_state = read_varint(cursor)?;

            info!(
                "Handshake: protocol={}, next_state={}",
                protocol_version, next_state
            );

            self.state = match next_state {
                1 => ConnectionState::Status,
                2 => ConnectionState::Login,
                _ => {
                    warn!("Unknown next state: {}", next_state);
                    return Ok(());
                }
            };
        }
        Ok(())
    }

    async fn handle_status(
        &mut self,
        packet_id: i32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> anyhow::Result<bool> {
        match packet_id {
            0 => {
                // Status Request
                info!("Status request");
                let status = ServerStatus {
                    version: Version {
                        name: mc_packets::PROTOCOL_NAME.to_string(),
                        protocol: mc_packets::PROTOCOL_VERSION,
                    },
                    players: Players {
                        max: 100,
                        online: 0,
                        sample: vec![],
                    },
                    description: Description {
                        text: "A Rust Minecraft Server".to_string(),
                    },
                    enforces_secure_chat: false,
                };
                let json = serde_json::to_string(&status)?;
                let mut data = Vec::new();
                json.encode(&mut data)?;
                self.send_packet(0, &data).await?;
            }
            1 => {
                // Ping
                let time = i64::decode(cursor)?;
                let mut data = Vec::new();
                time.encode(&mut data)?;
                self.send_packet(1, &data).await?;
                return Ok(false); // Close after ping
            }
            _ => {}
        }
        Ok(true)
    }

    async fn handle_login(
        &mut self,
        packet_id: i32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> anyhow::Result<()> {
        match packet_id {
            0 => {
                // Login Start: String (name), UUID
                let name = String::decode(cursor)?;
                let uuid = Uuid::decode(cursor)?;

                info!("Login from: {} (uuid: {:032x})", name, uuid.0);

                // For offline mode, generate UUID from name
                let player_uuid = offline_uuid(&name);
                self.player_name = Some(name.clone());
                self.player_uuid = Some(player_uuid);

                // Send Login Success (packet 0x02)
                // Fields: UUID, String (username), VarInt (properties count)
                // Note: No "strict error handling" field in 1.21+
                let mut data = Vec::new();
                Uuid(player_uuid).encode(&mut data)?;
                name.encode(&mut data)?;
                write_varint(&mut data, 0)?; // 0 properties

                self.send_packet(2, &data).await?;
                info!("Sent Login Success, waiting for Login Acknowledged");
            }
            3 => {
                // Login Acknowledged - client is ready for configuration
                info!("Login Acknowledged, transitioning to Configuration");
                self.state = ConnectionState::Configuration;

                // Immediately send Known Packs
                self.send_known_packs().await?;
            }
            _ => {
                debug!("Unknown login packet: {}", packet_id);
            }
        }
        Ok(())
    }

    async fn send_known_packs(&mut self) -> anyhow::Result<()> {
        // Clientbound Known Packs (0x0E in configuration)
        let mut data = Vec::new();
        write_varint(&mut data, 1)?; // 1 pack
        "minecraft".encode(&mut data)?; // namespace
        "core".encode(&mut data)?; // id
        "1.21".encode(&mut data)?; // version

        self.send_packet(14, &data).await?;
        debug!("Sent Known Packs");
        Ok(())
    }

    async fn handle_configuration(
        &mut self,
        packet_id: i32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> anyhow::Result<()> {
        debug!("Configuration packet: {}", packet_id);

        match packet_id {
            0 => {
                // Client Information - client settings
                debug!("Got Client Information");
            }
            2 => {
                // Custom Payload (plugin message)
                let channel = String::decode(cursor)?;
                debug!("Plugin message on channel: {}", channel);
            }
            3 => {
                // Finish Configuration (Acknowledge)
                info!("Client acknowledged configuration, transitioning to Play");
                self.state = ConnectionState::Play;
                self.send_play_login().await?;
            }
            7 => {
                // Select Known Packs response
                let count = read_varint(cursor)?;
                debug!("Client selected {} known packs", count);

                // Send Registry Data and Finish Configuration
                self.send_registry_data().await?;
                self.send_finish_configuration().await?;
            }
            _ => {
                debug!("Unknown configuration packet: {}", packet_id);
            }
        }

        Ok(())
    }

    async fn send_registry_data(&mut self) -> anyhow::Result<()> {
        // Send all required registries for 1.21+
        self.send_dimension_type_registry().await?;
        self.send_biome_registry().await?;
        self.send_damage_type_registry().await?;

        // Animal variants (required in 1.21+)
        self.send_cat_variant_registry().await?;
        self.send_chicken_variant_registry().await?;
        self.send_cow_variant_registry().await?;
        self.send_frog_variant_registry().await?;
        self.send_pig_variant_registry().await?;
        self.send_wolf_variant_registry().await?;
        self.send_wolf_sound_variant_registry().await?;
        self.send_zombie_nautilus_variant_registry().await?;
        self.send_painting_variant_registry().await?;

        Ok(())
    }

    async fn send_dimension_type_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:dimension_type".encode(&mut data)?;

        // 1 entry
        write_varint(&mut data, 1)?;

        // Entry: identifier, has_data (bool), data (NBT if has_data)
        "minecraft:overworld".encode(&mut data)?;
        true.encode(&mut data)?; // has data

        // NBT compound for dimension type
        // This is simplified - real implementation needs proper NBT
        let nbt = create_dimension_type_nbt();
        data.extend_from_slice(&nbt);

        self.send_packet(7, &data).await?;
        debug!("Sent dimension_type registry");
        Ok(())
    }

    async fn send_biome_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:worldgen/biome".encode(&mut data)?;

        write_varint(&mut data, 1)?; // 1 entry
        "minecraft:plains".encode(&mut data)?;
        true.encode(&mut data)?;

        let nbt = create_biome_nbt();
        data.extend_from_slice(&nbt);

        self.send_packet(7, &data).await?;
        debug!("Sent biome registry");
        Ok(())
    }

    async fn send_damage_type_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:damage_type".encode(&mut data)?;

        write_varint(&mut data, DAMAGE_TYPES.len() as i32)?;

        for dt in DAMAGE_TYPES {
            format!("minecraft:{}", dt.name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt =
                create_damage_type_nbt_full(dt.message_id, dt.exhaustion, dt.scaling, dt.effects);
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!(
            "Sent damage_type registry with {} entries",
            DAMAGE_TYPES.len()
        );
        Ok(())
    }

    async fn send_cat_variant_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:cat_variant".encode(&mut data)?;

        let variants = [
            "all_black",
            "black",
            "british_shorthair",
            "calico",
            "jellie",
            "persian",
            "ragdoll",
            "red",
            "siamese",
            "tabby",
            "white",
        ];
        write_varint(&mut data, variants.len() as i32)?;

        for name in variants {
            format!("minecraft:{}", name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt = create_asset_variant_nbt(&format!("minecraft:entity/cat/{}", name));
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!("Sent cat_variant registry");
        Ok(())
    }

    async fn send_chicken_variant_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:chicken_variant".encode(&mut data)?;

        let variants = ["cold", "temperate", "warm"];
        write_varint(&mut data, variants.len() as i32)?;

        for name in variants {
            format!("minecraft:{}", name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt =
                create_asset_variant_nbt(&format!("minecraft:entity/chicken/{}_chicken", name));
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!("Sent chicken_variant registry");
        Ok(())
    }

    async fn send_cow_variant_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:cow_variant".encode(&mut data)?;

        let variants = ["cold", "temperate", "warm"];
        write_varint(&mut data, variants.len() as i32)?;

        for name in variants {
            format!("minecraft:{}", name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt = create_asset_variant_nbt(&format!("minecraft:entity/cow/{}_cow", name));
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!("Sent cow_variant registry");
        Ok(())
    }

    async fn send_frog_variant_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:frog_variant".encode(&mut data)?;

        let variants = ["cold", "temperate", "warm"];
        write_varint(&mut data, variants.len() as i32)?;

        for name in variants {
            format!("minecraft:{}", name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt = create_asset_variant_nbt(&format!("minecraft:entity/frog/{}_frog", name));
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!("Sent frog_variant registry");
        Ok(())
    }

    async fn send_pig_variant_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:pig_variant".encode(&mut data)?;

        let variants = ["cold", "temperate", "warm"];
        write_varint(&mut data, variants.len() as i32)?;

        for name in variants {
            format!("minecraft:{}", name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt = create_asset_variant_nbt(&format!("minecraft:entity/pig/{}_pig", name));
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!("Sent pig_variant registry");
        Ok(())
    }

    async fn send_wolf_variant_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:wolf_variant".encode(&mut data)?;

        let variants = [
            "ashen", "black", "chestnut", "pale", "rusty", "snowy", "spotted", "striped", "woods",
        ];
        write_varint(&mut data, variants.len() as i32)?;

        for name in variants {
            format!("minecraft:{}", name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt = create_wolf_variant_nbt(name);
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!("Sent wolf_variant registry");
        Ok(())
    }

    async fn send_wolf_sound_variant_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:wolf_sound_variant".encode(&mut data)?;

        let variants = ["angry", "big", "classic", "cute", "grumpy", "puglin", "sad"];
        write_varint(&mut data, variants.len() as i32)?;

        for name in variants {
            format!("minecraft:{}", name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt = create_wolf_sound_variant_nbt();
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!("Sent wolf_sound_variant registry");
        Ok(())
    }

    async fn send_zombie_nautilus_variant_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:zombie_nautilus_variant".encode(&mut data)?;

        let variants = ["temperate", "warm"];
        write_varint(&mut data, variants.len() as i32)?;

        for name in variants {
            format!("minecraft:{}", name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt = create_asset_variant_nbt("minecraft:entity/nautilus/zombie_nautilus");
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!("Sent zombie_nautilus_variant registry");
        Ok(())
    }

    async fn send_painting_variant_registry(&mut self) -> anyhow::Result<()> {
        let mut data = Vec::new();
        "minecraft:painting_variant".encode(&mut data)?;

        // All painting variants from the data files
        let variants = [
            ("alban", 1, 1),
            ("aztec", 1, 1),
            ("aztec2", 1, 1),
            ("bomb", 1, 1),
            ("bouquet", 3, 3),
            ("burning_skull", 4, 4),
            ("bust", 2, 2),
            ("cavebird", 3, 3),
            ("changing", 4, 2),
            ("cotan", 3, 3),
            ("courbet", 2, 1),
            ("creebet", 2, 1),
            ("donkey_kong", 4, 3),
            ("earth", 2, 2),
            ("endboss", 3, 3),
            ("fern", 3, 3),
            ("fighters", 4, 2),
            ("finding", 4, 2),
            ("fire", 2, 2),
            ("graham", 1, 2),
            ("humble", 2, 2),
            ("kebab", 1, 2),
            ("lowmist", 4, 2),
            ("match", 2, 2),
            ("meditative", 1, 1),
            ("orb", 4, 4),
            ("owlemons", 3, 3),
            ("passage", 4, 2),
            ("pigscene", 4, 4),
            ("plant", 1, 1),
            ("pointer", 4, 4),
            ("pond", 3, 4),
            ("pool", 2, 1),
            ("prairie_ride", 1, 2),
            ("sea", 2, 1),
            ("skeleton", 4, 3),
            ("skull_and_roses", 2, 2),
            ("stage", 2, 2),
            ("sunflowers", 3, 3),
            ("sunset", 2, 1),
            ("tides", 3, 3),
            ("unpacked", 4, 4),
            ("void", 2, 2),
            ("wanderer", 1, 2),
            ("wasteland", 1, 1),
            ("water", 2, 2),
            ("wind", 2, 2),
            ("wither", 2, 2),
            // Backrooms paintings
            ("backyard", 3, 4),
            ("baroque", 2, 2),
            ("endboss", 3, 3),
        ];

        // Deduplicate
        let mut seen = std::collections::HashSet::new();
        let unique_variants: Vec<_> = variants
            .iter()
            .filter(|(n, _, _)| seen.insert(*n))
            .collect();

        write_varint(&mut data, unique_variants.len() as i32)?;

        for (name, width, height) in unique_variants {
            format!("minecraft:{}", name).encode(&mut data)?;
            true.encode(&mut data)?;
            let nbt = create_painting_variant_nbt(name, *width, *height);
            data.extend_from_slice(&nbt);
        }

        self.send_packet(7, &data).await?;
        debug!("Sent painting_variant registry");
        Ok(())
    }

    async fn send_finish_configuration(&mut self) -> anyhow::Result<()> {
        // Finish Configuration (0x03) - no data
        self.send_packet(3, &[]).await?;
        debug!("Sent Finish Configuration");
        Ok(())
    }

    async fn send_play_login(&mut self) -> anyhow::Result<()> {
        // Login (Play) packet (0x2C for 1.21.x)
        // This is a complex packet with many fields

        let mut data = Vec::new();

        // Entity ID (Int)
        WriteBytesExt::write_i32::<BigEndian>(&mut data, self.entity_id)?;

        // Is Hardcore (Boolean)
        false.encode(&mut data)?;

        // Dimension Count (VarInt) + Dimension Names (Array of Identifier)
        write_varint(&mut data, 1)?; // 1 dimension
        "minecraft:overworld".encode(&mut data)?;

        // Max Players (VarInt) - ignored by client
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
        write_varint(&mut data, 0)?; // First dimension type

        // Dimension Name (Identifier)
        "minecraft:overworld".encode(&mut data)?;

        // Hashed Seed (Long)
        WriteBytesExt::write_i64::<BigEndian>(&mut data, 0)?;

        // Game Mode (Unsigned Byte)
        WriteBytesExt::write_u8(&mut data, 1)?; // Creative

        // Previous Game Mode (Byte)
        WriteBytesExt::write_i8(&mut data, -1)?; // None

        // Is Debug (Boolean)
        false.encode(&mut data)?;

        // Is Flat (Boolean)
        true.encode(&mut data)?; // Superflat!

        // Has Death Location (Boolean)
        false.encode(&mut data)?;

        // Portal Cooldown (VarInt)
        write_varint(&mut data, 0)?;

        // Sea Level (VarInt)
        write_varint(&mut data, 63)?;

        // Enforces Secure Chat (Boolean)
        false.encode(&mut data)?;

        self.send_packet(PlayLogin::ID, &data).await?;
        info!("Sent Play Login packet");

        // Send initial position
        self.send_player_position().await?;

        // Send game event to start waiting for chunks
        self.send_game_event_start_waiting().await?;

        // Send chunks around spawn
        self.send_spawn_chunks().await?;

        // Send Set Center Chunk
        self.send_set_center_chunk(0, 0).await?;

        // Start keep-alive task
        self.start_keepalive().await?;

        Ok(())
    }

    async fn send_player_position(&mut self) -> anyhow::Result<()> {
        // Synchronize Player Position (0x42)
        let mut data = Vec::new();

        // Teleport ID (VarInt)
        write_varint(&mut data, 1)?;

        // X, Y, Z (Double)
        WriteBytesExt::write_f64::<BigEndian>(&mut data, 0.0)?; // X
        WriteBytesExt::write_f64::<BigEndian>(&mut data, 4.0)?; // Y (above ground)
        WriteBytesExt::write_f64::<BigEndian>(&mut data, 0.0)?; // Z

        // Velocity X, Y, Z (Double) - new in 1.21.2+
        WriteBytesExt::write_f64::<BigEndian>(&mut data, 0.0)?;
        WriteBytesExt::write_f64::<BigEndian>(&mut data, 0.0)?;
        WriteBytesExt::write_f64::<BigEndian>(&mut data, 0.0)?;

        // Yaw, Pitch (Float)
        WriteBytesExt::write_f32::<BigEndian>(&mut data, 0.0)?;
        WriteBytesExt::write_f32::<BigEndian>(&mut data, 0.0)?;

        // Flags (Int bitfield) - 0 means all absolute
        WriteBytesExt::write_i32::<BigEndian>(&mut data, 0)?;

        self.send_packet(PlayerPosition::ID, &data).await?;
        debug!("Sent Player Position");
        Ok(())
    }

    async fn send_game_event_start_waiting(&mut self) -> anyhow::Result<()> {
        // Game Event (0x22)
        let mut data = Vec::new();
        WriteBytesExt::write_u8(&mut data, 13)?; // Event: Start waiting for level chunks
        WriteBytesExt::write_f32::<BigEndian>(&mut data, 0.0)?; // Value (unused for this event)

        self.send_packet(GameEvent::ID, &data).await?;
        debug!("Sent Game Event: Start waiting for chunks");
        Ok(())
    }

    async fn send_set_center_chunk(&mut self, x: i32, z: i32) -> anyhow::Result<()> {
        // Set Center Chunk (0x4F)
        let mut data = Vec::new();
        write_varint(&mut data, x)?;
        write_varint(&mut data, z)?;

        self.send_packet(SetChunkCacheCenter::ID, &data).await?;
        debug!("Sent Set Center Chunk");
        Ok(())
    }

    async fn send_spawn_chunks(&mut self) -> anyhow::Result<()> {
        // Send a 3x3 grid of chunks around spawn
        for cx in -1..=1 {
            for cz in -1..=1 {
                self.send_chunk(cx, cz).await?;
            }
        }
        Ok(())
    }

    async fn send_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> anyhow::Result<()> {
        // Chunk Data and Update Light (0x27)
        // Based on decompiled aer.java (level_chunk_with_light)
        let mut data = Vec::new();

        // Chunk X, Z (Int)
        WriteBytesExt::write_i32::<BigEndian>(&mut data, chunk_x)?;
        WriteBytesExt::write_i32::<BigEndian>(&mut data, chunk_z)?;

        // ========== Chunk Data (aeq) ==========

        // Heightmaps: Map<euq.a, long[]>
        // Format: VarInt count, then for each: VarInt enum_id + VarInt array_len + longs
        // Try with empty heightmaps first to test if the issue is elsewhere
        write_varint(&mut data, 0)?; // 0 heightmap entries (empty map)

        // Chunk section data (byte array with VarInt length prefix)
        let chunk_data = create_superflat_chunk_data();
        write_varint(&mut data, chunk_data.len() as i32)?;
        data.extend_from_slice(&chunk_data);

        // Block Entities (VarInt count + array)
        write_varint(&mut data, 0)?; // No block entities

        // ========== Light Data (aev) ==========
        // BitSet format: VarInt count of longs, then longs

        // Sky light mask (BitSet) - which sections have sky light
        write_varint(&mut data, 0)?; // empty bitset (0 longs)

        // Block light mask (BitSet)
        write_varint(&mut data, 0)?;

        // Empty sky light mask (BitSet) - which sections are fully lit
        write_varint(&mut data, 0)?;

        // Empty block light mask (BitSet)
        write_varint(&mut data, 0)?;

        // Sky Light Arrays (List<byte[2048]>)
        write_varint(&mut data, 0)?; // 0 arrays

        // Block Light Arrays (List<byte[2048]>)
        write_varint(&mut data, 0)?; // 0 arrays

        // Debug: print chunk packet data breakdown
        info!("Chunk packet data size: {} bytes", data.len());
        info!("First 50 bytes: {:02x?}", &data[..50.min(data.len())]);
        info!(
            "Bytes 8-20 (after coords): {:02x?}",
            &data[8..20.min(data.len())]
        );

        self.send_packet(LevelChunkWithLight::ID, &data).await?;
        debug!("Sent chunk ({}, {})", chunk_x, chunk_z);
        Ok(())
    }

    async fn start_keepalive(&mut self) -> anyhow::Result<()> {
        // Send initial keep-alive
        let mut data = Vec::new();
        WriteBytesExt::write_i64::<BigEndian>(&mut data, 0)?;
        self.send_packet(ClientboundKeepAlive::ID, &data).await?;
        Ok(())
    }

    async fn handle_play(
        &mut self,
        packet_id: i32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> anyhow::Result<()> {
        match packet_id {
            AcceptTeleportation::ID => {
                let teleport_id = read_varint(cursor)?;
                debug!("Client accepted teleport: {}", teleport_id);
            }
            ServerboundKeepAlive::ID => {
                let ka_id = i64::decode(cursor)?;
                debug!("Keep alive response: {}", ka_id);
            }
            MovePlayerPos::ID
            | MovePlayerPosRot::ID
            | MovePlayerRot::ID
            | MovePlayerStatusOnly::ID => {
                // Player position/rotation packets - ignore for now
            }
            _ => {
                debug!("Play packet: 0x{:02X}", packet_id);
            }
        }
        Ok(())
    }
}

/// Convert JSON value to network NBT (nameless root compound)
fn json_to_network_nbt(json: &serde_json::Value) -> Vec<u8> {
    // Network NBT format: the root compound has no name
    // fastnbt serializes with a name, so we need to strip it
    let nbt_bytes = fastnbt::to_bytes(json).unwrap();

    // fastnbt output: 0x0A (compound) + 2-byte name length + name + contents + 0x00
    // For network NBT we need: 0x0A (compound) + contents + 0x00
    // The standard NBT has an empty name (""), so it's: 0x0A 0x00 0x00 + contents + 0x00
    // We need to skip the name length bytes

    // Actually fastnbt writes: 0x0A + name_len (2 bytes) + name + payload + 0x00
    // For nameless NBT we write: 0x0A + payload + 0x00

    if nbt_bytes.len() >= 3 && nbt_bytes[0] == 0x0A {
        // Read the name length
        let name_len = u16::from_be_bytes([nbt_bytes[1], nbt_bytes[2]]) as usize;
        let skip = 3 + name_len; // tag type + name length + name

        let mut result = Vec::with_capacity(nbt_bytes.len() - name_len - 2);
        result.push(0x0A); // Compound tag (nameless)
        result.extend_from_slice(&nbt_bytes[skip..]); // Contents including final 0x00
        result
    } else {
        nbt_bytes
    }
}

/// Create dimension type NBT from JSON (1.21+ format)
fn create_dimension_type_nbt() -> Vec<u8> {
    // Use the actual 1.21 dimension type format
    let json = serde_json::json!({
        "ambient_light": 0.0,
        "coordinate_scale": 1.0,
        "has_ceiling": false,
        "has_skylight": true,
        "height": 384,
        "infiniburn": "#minecraft:infiniburn_overworld",
        "logical_height": 384,
        "min_y": -64,
        "monster_spawn_block_light_limit": 0,
        "monster_spawn_light_level": {
            "type": "minecraft:uniform",
            "max_inclusive": 7,
            "min_inclusive": 0
        }
    });

    json_to_network_nbt(&json)
}

fn create_biome_nbt() -> Vec<u8> {
    let json = serde_json::json!({
        "has_precipitation": true,
        "temperature": 0.8,
        "downfall": 0.4,
        "effects": {
            "water_color": 0x3F76E4i32
        }
    });
    json_to_network_nbt(&json)
}

fn create_damage_type_nbt_full(
    message_id: &str,
    exhaustion: f32,
    scaling: &str,
    effects: &str,
) -> Vec<u8> {
    let mut json = serde_json::json!({
        "message_id": message_id,
        "scaling": scaling,
        "exhaustion": exhaustion
    });
    // Only add effects if not default "hurt"
    if effects != "hurt" {
        json["effects"] = serde_json::json!(effects);
    }
    json_to_network_nbt(&json)
}

/// Create NBT for asset-based variants (cat, chicken, cow, frog, pig, zombie_nautilus)
fn create_asset_variant_nbt(asset_id: &str) -> Vec<u8> {
    let json = serde_json::json!({
        "asset_id": asset_id,
        "spawn_conditions": [
            { "priority": 0 }
        ]
    });
    json_to_network_nbt(&json)
}

/// Create NBT for wolf variants (has assets compound with angry, tame, wild)
fn create_wolf_variant_nbt(name: &str) -> Vec<u8> {
    let json = serde_json::json!({
        "assets": {
            "angry": format!("minecraft:entity/wolf/{}_angry", name),
            "tame": format!("minecraft:entity/wolf/{}_tame", name),
            "wild": format!("minecraft:entity/wolf/{}", name)
        },
        "spawn_conditions": [
            { "priority": 0 }
        ]
    });
    json_to_network_nbt(&json)
}

/// Create NBT for wolf sound variants
fn create_wolf_sound_variant_nbt() -> Vec<u8> {
    let json = serde_json::json!({
        "ambient_sound": "minecraft:entity.wolf.ambient",
        "death_sound": "minecraft:entity.wolf.death",
        "growl_sound": "minecraft:entity.wolf.growl",
        "hurt_sound": "minecraft:entity.wolf.hurt",
        "pant_sound": "minecraft:entity.wolf.pant",
        "whine_sound": "minecraft:entity.wolf.whine"
    });
    json_to_network_nbt(&json)
}

/// Create NBT for painting variants
fn create_painting_variant_nbt(name: &str, width: i32, height: i32) -> Vec<u8> {
    let json = serde_json::json!({
        "asset_id": format!("minecraft:{}", name),
        "width": width,
        "height": height,
        "title": {
            "translate": format!("painting.minecraft.{}.title", name),
            "color": "yellow"
        },
        "author": {
            "translate": format!("painting.minecraft.{}.author", name),
            "color": "gray"
        }
    });
    json_to_network_nbt(&json)
}

/// Create heightmap long array for a superflat world
/// Heightmaps use 9 bits per entry (for heights 0-384 with min_y=-64)
/// 256 entries (16x16), packed into longs
fn create_heightmap_longs() -> Vec<i64> {
    // For superflat, the surface is at Y=4 (bedrock at 0, dirt 1-2, grass at 3)
    // Heightmap stores "highest block + 1", so value = 4
    // But with min_y = -64, we need to store (4 - (-64)) = 68

    // 9 bits per entry, 256 entries = ceil(256 * 9 / 64) = 36 longs
    // Actually, Minecraft packs them without spanning longs, so:
    // 64 / 9 = 7 entries per long, 256 / 7 = 37 longs needed
    let entries_per_long = 64 / 9; // 7
    let num_longs = (256 + entries_per_long - 1) / entries_per_long; // 37

    let height_value: u64 = 68; // Y=4 stored as 4 - (-64) = 68

    let mut longs = Vec::with_capacity(num_longs);
    let mut entry_idx = 0;

    for _ in 0..num_longs {
        let mut long_val: u64 = 0;
        for i in 0..entries_per_long {
            if entry_idx < 256 {
                long_val |= height_value << (i * 9);
                entry_idx += 1;
            }
        }
        longs.push(long_val as i64);
    }

    longs
}

/// Create superflat chunk data (24 sections from Y=-64 to Y=320)
fn create_superflat_chunk_data() -> Vec<u8> {
    let mut data = Vec::new();

    // 24 chunk sections (-64 to 320, each 16 blocks tall)
    for section_y in 0..24 {
        // Block count (Short) - non-air blocks
        let block_count: i16 = if section_y == 4 { 16 * 16 * 4 } else { 0 }; // Section at Y=0 has 4 layers
        data.extend_from_slice(&block_count.to_be_bytes());

        // Block states (PalettedContainer)
        if section_y == 4 {
            // This section contains the superflat layers at Y=0-3 (bedrock, dirt, dirt, grass)
            write_superflat_section(&mut data);
        } else {
            // Air section (single value palette)
            // From SingleValuePalette.write(): just writes VarInt(global_id)
            // From PalettedContainer.Data.write():
            //   1. writeByte(bits) = 0 for single value
            //   2. palette.write() = VarInt(global_id) for single value
            //   3. writeFixedSizeLongArray() = nothing for 0 bits (ZeroBitStorage has 0 longs)
            data.push(0); // Bits per entry = 0 (single value)
            write_varint_to_vec(&mut data, 0); // Palette: air (block state 0)
            // No data array for 0-bit storage!
        }

        // Biomes (PalettedContainer) - single value (plains)
        // Same format: 0 bits, VarInt global_id, no data
        data.push(0); // Bits per entry = 0
        write_varint_to_vec(&mut data, 0); // Palette: plains biome (ID 0)
        // No data array for 0-bit storage!
    }

    data
}

fn write_superflat_section(data: &mut Vec<u8>) {
    // For superflat: Y=0 bedrock, Y=1-2 dirt, Y=3 grass_block, Y=4-15 air
    // Use a palette with these blocks
    //
    // From PalettedContainer.Data.write():
    //   1. writeByte(bits)
    //   2. palette.write() - for HashMapPalette: VarInt(size) + size * VarInt(global_id)
    //   3. writeFixedSizeLongArray() - NO length prefix! Just raw longs

    // Bits per entry - need at least 2 bits for 4 block types
    // Minecraft uses palette when bits <= 8, direct when bits > 8
    data.push(4); // 4 bits per entry (uses palette)

    // Palette: HashMapPalette.write() = VarInt(size) + VarInt(id) for each entry
    write_varint_to_vec(data, 4); // 4 palette entries

    // Palette entries (global block state IDs)
    write_varint_to_vec(data, 0); // 0: air
    write_varint_to_vec(data, 79); // 1: bedrock (approximate ID)
    write_varint_to_vec(data, 10); // 2: dirt (approximate ID)
    write_varint_to_vec(data, 9); // 3: grass_block (approximate ID)

    // Data array: 16x16x16 = 4096 blocks, 4 bits each
    // From SimpleBitStorage: valuesPerLong = 64 / 4 = 16
    // requiredLength = (4096 + 16 - 1) / 16 = 256 longs
    // NO length prefix - client computes size from bits and entry count

    // Write longs directly without length prefix
    for y in 0..16 {
        for _z in 0..16 {
            // Each long holds 16 blocks (one row of X values)
            let mut long_val: u64 = 0;
            for i in 0..16 {
                let block_idx = match y {
                    0 => 1,     // bedrock
                    1 | 2 => 2, // dirt
                    3 => 3,     // grass_block
                    _ => 0,     // air
                };
                long_val |= (block_idx as u64) << (i * 4);
            }
            data.extend_from_slice(&long_val.to_be_bytes());
        }
    }
}

fn write_varint_to_vec(buf: &mut Vec<u8>, value: i32) {
    write_varint(buf, value).unwrap();
}

static ENTITY_ID_COUNTER: AtomicI64 = AtomicI64::new(1);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_server=debug".parse()?),
        )
        .init();

    let addr = "0.0.0.0:25565";
    let listener = TcpListener::bind(addr).await?;
    info!(
        "Minecraft server listening on {} (version {})",
        addr,
        mc_packets::PROTOCOL_NAME
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Connection from {}", addr);

        let entity_id = ENTITY_ID_COUNTER.fetch_add(1, Ordering::Relaxed) as i32;

        tokio::spawn(async move {
            let mut conn = Connection::new(stream, entity_id);
            if let Err(e) = conn.handle().await {
                warn!("Connection error: {}", e);
            }
        });
    }
}
