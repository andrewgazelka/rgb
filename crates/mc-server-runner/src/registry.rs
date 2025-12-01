#![allow(dead_code)]
use mc_protocol::{Encode, nbt, write_varint};

/// Damage type definition for registry
pub struct DamageType {
    pub name: &'static str,
    pub message_id: &'static str,
    pub exhaustion: f32,
    pub scaling: &'static str,
    pub effects: &'static str,
}

pub const DAMAGE_TYPES: &[DamageType] = &[
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

// Registry Data Generation

pub fn create_dimension_type_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:dimension_type".to_string().encode(&mut data)?;

    write_varint(&mut data, 1)?;
    "minecraft:overworld".to_string().encode(&mut data)?;
    true.encode(&mut data)?;

    // DimensionType format for 1.21.11-pre3:
    // - has_fixed_time (optional, default false)
    // - has_skylight
    // - has_ceiling
    // - coordinate_scale
    // - min_y
    // - height
    // - logical_height
    // - infiniburn (block tag)
    // - ambient_light
    // - monster_spawn_light_level (IntProvider)
    // - monster_spawn_block_light_limit
    // - skybox (optional: "none", "overworld", "end")
    // - cardinal_light (optional: "default", "nether")
    // - attributes (EnvironmentAttributeMap with sky_color, fog_color, etc.)
    // - timelines (optional, HolderSet)
    //
    // Sky color for temperature 0.8: calculateSkyColor(0.8) = 0x78A7FF (7907327)
    let nbt_data = nbt! {
        "has_skylight" => true,
        "has_ceiling" => false,
        "coordinate_scale" => 1.0f64,
        "min_y" => -64i32,
        "height" => 384i32,
        "logical_height" => 384i32,
        "infiniburn" => "#minecraft:infiniburn_overworld",
        "ambient_light" => 0.0f32,
        "monster_spawn_light_level" => nbt! {
            "type" => "minecraft:uniform",
            "min_inclusive" => 0i32,
            "max_inclusive" => 7i32,
        },
        "monster_spawn_block_light_limit" => 0i32,
        "skybox" => "overworld",
        "attributes" => nbt! {
            "minecraft:visual/fog_color" => 0xC0D8FFi32,
            "minecraft:visual/sky_color" => 0x78A7FFi32,
            // Cloud color is ARGB - 0xCC alpha (80%) with white (0xFFFFFF)
            "minecraft:visual/cloud_color" => 0xCCFFFFFFu32 as i32,
            "minecraft:visual/cloud_height" => 192.33f32,
        },
    };
    data.extend_from_slice(&nbt_data.to_network_bytes());

    Ok(data)
}

pub fn create_biome_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:worldgen/biome".to_string().encode(&mut data)?;

    write_varint(&mut data, 1)?;
    "minecraft:plains".to_string().encode(&mut data)?;
    true.encode(&mut data)?;

    // Biome format for 1.21.11-pre3:
    // - ClimateSettings: has_precipitation, temperature, downfall
    // - attributes (optional): EnvironmentAttributeMap for positional overrides
    //   (sky_color is now in dimension type, biome can override based on temperature)
    // - effects: BiomeSpecialEffects with water_color
    let nbt_data = nbt! {
        "has_precipitation" => true,
        "temperature" => 0.8f32,
        "downfall" => 0.4f32,
        "effects" => nbt! {
            "water_color" => 0x3F76E4i32,
        },
    };
    data.extend_from_slice(&nbt_data.to_network_bytes());

    Ok(data)
}

pub fn create_damage_type_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:damage_type".to_string().encode(&mut data)?;

    write_varint(&mut data, DAMAGE_TYPES.len() as i32)?;

    for dt in DAMAGE_TYPES {
        format!("minecraft:{}", dt.name).encode(&mut data)?;
        true.encode(&mut data)?;

        let mut nbt_data = nbt! {
            "message_id" => dt.message_id,
            "scaling" => dt.scaling,
            "exhaustion" => dt.exhaustion,
        };
        if dt.effects != "hurt" {
            nbt_data.insert("effects", dt.effects);
        }
        data.extend_from_slice(&nbt_data.to_network_bytes());
    }

    Ok(data)
}

/// Create NBT for asset-based variants (cat, chicken, cow, frog, pig)
/// Network codec only includes asset_id, NOT spawn_conditions
fn create_asset_variant_nbt(asset_id: &str) -> Vec<u8> {
    let nbt_data = nbt! {
        "asset_id" => asset_id,
    };
    nbt_data.to_network_bytes()
}

/// Create NBT for wolf variant
/// Network codec includes assets compound with angry/tame/wild textures
fn create_wolf_variant_nbt(name: &str) -> Vec<u8> {
    let nbt_data = nbt! {
        "assets" => nbt! {
            "angry" => format!("minecraft:entity/wolf/{}_angry", name),
            "tame" => format!("minecraft:entity/wolf/{}_tame", name),
            "wild" => format!("minecraft:entity/wolf/{}", name),
        },
    };
    nbt_data.to_network_bytes()
}

/// Create NBT for wolf sound variant
fn create_wolf_sound_variant_nbt() -> Vec<u8> {
    let nbt_data = nbt! {
        "ambient_sound" => "minecraft:entity.wolf.ambient",
        "death_sound" => "minecraft:entity.wolf.death",
        "growl_sound" => "minecraft:entity.wolf.growl",
        "hurt_sound" => "minecraft:entity.wolf.hurt",
        "pant_sound" => "minecraft:entity.wolf.pant",
        "whine_sound" => "minecraft:entity.wolf.whine",
    };
    nbt_data.to_network_bytes()
}

/// Create NBT for painting variant
fn create_painting_variant_nbt(name: &str, width: i32, height: i32) -> Vec<u8> {
    let nbt_data = nbt! {
        "width" => width,
        "height" => height,
        "asset_id" => format!("minecraft:{}", name),
    };
    nbt_data.to_network_bytes()
}

/// Create NBT for zombie nautilus variant
fn create_zombie_nautilus_variant_nbt() -> Vec<u8> {
    let nbt_data = nbt! {
        "asset_id" => "minecraft:entity/drowned/zombie_nautilus",
    };
    nbt_data.to_network_bytes()
}

pub fn create_cat_variant_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:cat_variant".to_string().encode(&mut data)?;

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
        data.extend_from_slice(&create_asset_variant_nbt(&format!(
            "minecraft:entity/cat/{}",
            name
        )));
    }

    Ok(data)
}

pub fn create_chicken_variant_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:chicken_variant".to_string().encode(&mut data)?;

    let variants = ["cold", "temperate", "warm"];
    write_varint(&mut data, variants.len() as i32)?;

    for name in variants {
        format!("minecraft:{}", name).encode(&mut data)?;
        true.encode(&mut data)?;
        data.extend_from_slice(&create_asset_variant_nbt(&format!(
            "minecraft:entity/chicken/{}_chicken",
            name
        )));
    }

    Ok(data)
}

pub fn create_cow_variant_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:cow_variant".to_string().encode(&mut data)?;

    let variants = ["cold", "temperate", "warm"];
    write_varint(&mut data, variants.len() as i32)?;

    for name in variants {
        format!("minecraft:{}", name).encode(&mut data)?;
        true.encode(&mut data)?;
        data.extend_from_slice(&create_asset_variant_nbt(&format!(
            "minecraft:entity/cow/{}_cow",
            name
        )));
    }

    Ok(data)
}

pub fn create_frog_variant_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:frog_variant".to_string().encode(&mut data)?;

    let variants = ["cold", "temperate", "warm"];
    write_varint(&mut data, variants.len() as i32)?;

    for name in variants {
        format!("minecraft:{}", name).encode(&mut data)?;
        true.encode(&mut data)?;
        data.extend_from_slice(&create_asset_variant_nbt(&format!(
            "minecraft:entity/frog/{}_frog",
            name
        )));
    }

    Ok(data)
}

pub fn create_pig_variant_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:pig_variant".to_string().encode(&mut data)?;

    let variants = ["cold", "temperate", "warm"];
    write_varint(&mut data, variants.len() as i32)?;

    for name in variants {
        format!("minecraft:{}", name).encode(&mut data)?;
        true.encode(&mut data)?;
        data.extend_from_slice(&create_asset_variant_nbt(&format!(
            "minecraft:entity/pig/{}_pig",
            name
        )));
    }

    Ok(data)
}

pub fn create_wolf_variant_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:wolf_variant".to_string().encode(&mut data)?;

    let variants = [
        "ashen", "black", "chestnut", "pale", "rusty", "snowy", "spotted", "striped", "woods",
    ];
    write_varint(&mut data, variants.len() as i32)?;

    for name in variants {
        format!("minecraft:{}", name).encode(&mut data)?;
        true.encode(&mut data)?;
        data.extend_from_slice(&create_wolf_variant_nbt(name));
    }

    Ok(data)
}

pub fn create_wolf_sound_variant_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:wolf_sound_variant"
        .to_string()
        .encode(&mut data)?;

    let variants = ["angry", "big", "classic", "cute", "grumpy", "puglin", "sad"];
    write_varint(&mut data, variants.len() as i32)?;

    for name in variants {
        format!("minecraft:{}", name).encode(&mut data)?;
        true.encode(&mut data)?;
        data.extend_from_slice(&create_wolf_sound_variant_nbt());
    }

    Ok(data)
}

pub fn create_zombie_nautilus_variant_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:zombie_nautilus_variant"
        .to_string()
        .encode(&mut data)?;

    let variants = ["temperate", "warm"];
    write_varint(&mut data, variants.len() as i32)?;

    for name in variants {
        format!("minecraft:{}", name).encode(&mut data)?;
        true.encode(&mut data)?;
        data.extend_from_slice(&create_zombie_nautilus_variant_nbt());
    }

    Ok(data)
}

pub fn create_painting_variant_registry() -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    "minecraft:painting_variant".to_string().encode(&mut data)?;

    let variants: &[(&str, i32, i32)] = &[
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
        ("backyard", 3, 4),
        ("baroque", 2, 2),
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
        data.extend_from_slice(&create_painting_variant_nbt(name, *width, *height));
    }

    Ok(data)
}
