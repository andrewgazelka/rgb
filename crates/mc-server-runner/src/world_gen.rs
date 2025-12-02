//! World generation - dune terrain

use byteorder::{BigEndian, WriteBytesExt};
use bytes::Bytes;
use mc_protocol::write_varint;
use rgb_ecs::World;
use tracing::info;

use crate::components::{ChunkData, ChunkLoaded, ChunkPos};

// ============================================================================
// Noise Implementation (Simplex-like)
// ============================================================================

/// Simple permutation table for noise
const PERM: [u8; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180,
];

fn hash(x: i32) -> u8 {
    PERM[(x & 255) as usize]
}

fn hash2(x: i32, y: i32) -> u8 {
    hash(x.wrapping_add(hash(y) as i32))
}

fn grad2(hash: u8, x: f64, y: f64) -> f64 {
    let h = hash & 7;
    let (u, v) = match h {
        0 => (x, y),
        1 => (y, x),
        2 => (-x, y),
        3 => (-y, x),
        4 => (x, -y),
        5 => (y, -x),
        6 => (-x, -y),
        7 => (-y, -x),
        _ => unreachable!(),
    };
    u + v
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

/// 2D Perlin noise, returns value in [-1, 1]
fn noise2d(x: f64, y: f64) -> f64 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let dx = x - x0 as f64;
    let dy = y - y0 as f64;

    let sx = fade(dx);
    let sy = fade(dy);

    let n00 = grad2(hash2(x0, y0), dx, dy);
    let n10 = grad2(hash2(x1, y0), dx - 1.0, dy);
    let n01 = grad2(hash2(x0, y1), dx, dy - 1.0);
    let n11 = grad2(hash2(x1, y1), dx - 1.0, dy - 1.0);

    let nx0 = lerp(n00, n10, sx);
    let nx1 = lerp(n01, n11, sx);

    lerp(nx0, nx1, sy)
}

/// Fractal Brownian Motion - multiple octaves of noise
fn fbm(x: f64, y: f64, octaves: u32, lacunarity: f64, persistence: f64) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += noise2d(x * frequency, y * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

/// Ridged noise - creates sharp ridges like dune crests
fn ridged_noise(x: f64, y: f64, octaves: u32) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut weight = 1.0;

    for _ in 0..octaves {
        let signal = 1.0 - noise2d(x * frequency, y * frequency).abs();
        let signal = signal * signal * weight;
        weight = (signal * 2.0).clamp(0.0, 1.0);
        value += signal * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value
}

// ============================================================================
// Dune Terrain Generation
// ============================================================================

/// Configuration for dune generation
struct DuneConfig {
    base_height: i32,
    max_dune_height: i32,
    ridge_scale: f64,
    wind_angle: f64,
    wind_stretch: f64,
}

impl Default for DuneConfig {
    fn default() -> Self {
        Self {
            base_height: 64,
            max_dune_height: 24,
            ridge_scale: 0.015,
            wind_angle: 0.4,
            wind_stretch: 2.5,
        }
    }
}

fn get_dune_height(world_x: i32, world_z: i32, config: &DuneConfig) -> i32 {
    let x = world_x as f64;
    let z = world_z as f64;

    let cos_a = config.wind_angle.cos();
    let sin_a = config.wind_angle.sin();
    let rx = x * cos_a - z * sin_a;
    let rz = x * sin_a + z * cos_a;

    let sx = rx * config.ridge_scale;
    let sz = rz * config.ridge_scale / config.wind_stretch;

    let ridges = ridged_noise(sx, sz, 3) * 0.6;
    let medium = fbm(sx * 2.0, sz * 2.0, 3, 2.0, 0.5) * 0.25;
    let ripples = fbm(sx * 8.0, sz * 4.0, 2, 2.0, 0.4) * 0.1;
    let micro = noise2d(sx * 16.0, sz * 16.0) * 0.05;

    let height_factor = ridges + medium + ripples + micro;
    let normalized = (height_factor + 0.5).clamp(0.0, 1.0);
    config.base_height + (normalized * config.max_dune_height as f64) as i32
}

fn get_block_at(_world_x: i32, world_y: i32, _world_z: i32, surface_height: i32) -> u16 {
    use mc_data::blocks;

    let depth = surface_height - world_y;

    if world_y <= 0 {
        blocks::BEDROCK.id()
    } else if world_y > surface_height {
        blocks::AIR.id()
    } else if depth == 0 {
        blocks::SAND.id()
    } else if depth < 4 {
        blocks::SAND.id()
    } else if depth < 8 {
        blocks::SANDSTONE.id()
    } else {
        blocks::STONE.id()
    }
}

// ============================================================================
// Chunk Encoding
// ============================================================================

fn create_dune_chunk(chunk_x: i32, chunk_z: i32) -> eyre::Result<Bytes> {
    let mut data = Vec::new();

    data.write_i32::<BigEndian>(chunk_x)?;
    data.write_i32::<BigEndian>(chunk_z)?;

    write_varint(&mut data, 0)?;

    let chunk_data = create_dune_sections(chunk_x, chunk_z);
    write_varint(&mut data, chunk_data.len() as i32)?;
    data.extend_from_slice(&chunk_data);

    write_varint(&mut data, 0)?;

    let mut sky_mask: u64 = 0;
    for i in 5..=25 {
        sky_mask |= 1u64 << i;
    }
    write_varint(&mut data, 1)?;
    data.write_i64::<BigEndian>(sky_mask as i64)?;

    write_varint(&mut data, 0)?;

    let mut empty_sky_mask: u64 = 0;
    for i in 0..5 {
        empty_sky_mask |= 1u64 << i;
    }
    write_varint(&mut data, 1)?;
    data.write_i64::<BigEndian>(empty_sky_mask as i64)?;

    let mut empty_block_mask: u64 = 0;
    for i in 0..26 {
        empty_block_mask |= 1u64 << i;
    }
    write_varint(&mut data, 1)?;
    data.write_i64::<BigEndian>(empty_block_mask as i64)?;

    let sky_section_count = (5..=25).count();
    write_varint(&mut data, sky_section_count as i32)?;

    let full_light = vec![0xFFu8; 2048];
    for _ in 0..sky_section_count {
        write_varint(&mut data, 2048)?;
        data.extend_from_slice(&full_light);
    }

    write_varint(&mut data, 0)?;

    Ok(Bytes::from(data))
}

fn create_dune_sections(chunk_x: i32, chunk_z: i32) -> Vec<u8> {
    use mc_data::blocks;

    let config = DuneConfig::default();
    let mut data = Vec::new();

    let mut heights = [[0i32; 16]; 16];
    for lz in 0..16 {
        for lx in 0..16 {
            let world_x = chunk_x * 16 + lx as i32;
            let world_z = chunk_z * 16 + lz as i32;
            heights[lz][lx] = get_dune_height(world_x, world_z, &config);
        }
    }

    for section_y in 0..24 {
        let section_min_y = (section_y as i32 - 4) * 16;

        let mut block_count: i16 = 0;
        let mut blocks_in_section = [[[0u16; 16]; 16]; 16];

        for local_y in 0..16 {
            let world_y = section_min_y + local_y as i32;
            for local_z in 0..16 {
                for local_x in 0..16 {
                    let surface_height = heights[local_z][local_x];
                    let world_x = chunk_x * 16 + local_x as i32;
                    let world_z = chunk_z * 16 + local_z as i32;

                    let block_id = get_block_at(world_x, world_y, world_z, surface_height);
                    blocks_in_section[local_y][local_z][local_x] = block_id;

                    if block_id != blocks::AIR.id() {
                        block_count += 1;
                    }
                }
            }
        }

        data.extend_from_slice(&block_count.to_be_bytes());

        if block_count == 0 {
            data.push(0);
            write_varint_vec(&mut data, blocks::AIR.id() as i32);
        } else {
            let mut palette: Vec<u16> = vec![blocks::AIR.id()];
            let mut palette_map = std::collections::HashMap::new();
            palette_map.insert(blocks::AIR.id(), 0u8);

            for local_y in 0..16 {
                for local_z in 0..16 {
                    for local_x in 0..16 {
                        let block_id = blocks_in_section[local_y][local_z][local_x];
                        if !palette_map.contains_key(&block_id) {
                            palette_map.insert(block_id, palette.len() as u8);
                            palette.push(block_id);
                        }
                    }
                }
            }

            let bits_per_entry = match palette.len() {
                1 => 0,
                2..=2 => 1,
                3..=4 => 2,
                5..=8 => 3,
                9..=16 => 4,
                17..=32 => 5,
                33..=64 => 6,
                65..=128 => 7,
                _ => 8,
            };

            if bits_per_entry == 0 {
                data.push(0);
                write_varint_vec(&mut data, palette[0] as i32);
            } else {
                let bits = bits_per_entry.max(4);
                data.push(bits);

                write_varint_vec(&mut data, palette.len() as i32);
                for block_id in &palette {
                    write_varint_vec(&mut data, *block_id as i32);
                }

                let entries_per_long = 64 / bits as usize;
                let mask = (1u64 << bits) - 1;

                let mut bit_buffer: u64 = 0;
                let mut entries_in_long = 0;

                for local_y in 0..16 {
                    for local_z in 0..16 {
                        for local_x in 0..16 {
                            let block_id = blocks_in_section[local_y][local_z][local_x];
                            let palette_idx = palette_map[&block_id] as u64;

                            let bit_offset = entries_in_long * bits as usize;
                            bit_buffer |= (palette_idx & mask) << bit_offset;
                            entries_in_long += 1;

                            if entries_in_long == entries_per_long {
                                data.extend_from_slice(&bit_buffer.to_be_bytes());
                                bit_buffer = 0;
                                entries_in_long = 0;
                            }
                        }
                    }
                }

                if entries_in_long > 0 {
                    data.extend_from_slice(&bit_buffer.to_be_bytes());
                }
            }
        }

        data.push(0);
        write_varint_vec(&mut data, 0);
    }

    data
}

fn write_varint_vec(buf: &mut Vec<u8>, value: i32) {
    write_varint(buf, value).expect("varint write");
}

/// Generate spawn chunks around origin
pub fn generate_spawn_chunks(world: &mut World, view_distance: i32) {
    for cx in -view_distance..=view_distance {
        for cz in -view_distance..=view_distance {
            let pos = ChunkPos::new(cx, cz);

            if let Ok(data) = create_dune_chunk(cx, cz) {
                // Use readable string name for dashboard visibility
                let name = format!("chunk({cx}, {cz})");
                let entity = world.entity_named(name.as_bytes());
                world.insert(entity, pos);
                world.insert(entity, ChunkData::new(data));
                world.insert(entity, ChunkLoaded);
            }
        }
    }

    info!(
        "Generated {} spawn chunks",
        (view_distance * 2 + 1) * (view_distance * 2 + 1)
    );
}
