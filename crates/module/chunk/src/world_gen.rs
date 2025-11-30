//! Chunk generation for superflat worlds

use byteorder::{BigEndian, WriteBytesExt};
use bytes::Bytes;
use mc_protocol::write_varint;

/// Create superflat chunk packet data (without packet ID)
pub fn create_superflat_chunk(chunk_x: i32, chunk_z: i32) -> eyre::Result<Bytes> {
    let mut data = Vec::new();

    // Chunk X, Z (Int)
    data.write_i32::<BigEndian>(chunk_x)?;
    data.write_i32::<BigEndian>(chunk_z)?;

    // Heightmaps: empty
    write_varint(&mut data, 0)?;

    // Chunk section data
    let chunk_data = create_superflat_chunk_sections();
    write_varint(&mut data, chunk_data.len() as i32)?;
    data.extend_from_slice(&chunk_data);

    // Block Entities
    write_varint(&mut data, 0)?;

    // Light Data
    // Sky light mask
    let mut sky_mask: u64 = 0;
    for i in 5..=25 {
        sky_mask |= 1u64 << i;
    }
    write_varint(&mut data, 1)?;
    data.write_i64::<BigEndian>(sky_mask as i64)?;

    // Block light mask
    write_varint(&mut data, 0)?;

    // Empty sky light mask
    let mut empty_sky_mask: u64 = 0;
    for i in 0..5 {
        empty_sky_mask |= 1u64 << i;
    }
    write_varint(&mut data, 1)?;
    data.write_i64::<BigEndian>(empty_sky_mask as i64)?;

    // Empty block light mask
    write_varint(&mut data, 0)?;

    // Sky Light Arrays
    let sky_section_count = (5..=25).count();
    write_varint(&mut data, sky_section_count as i32)?;

    let full_light = vec![0xFFu8; 2048];
    for _ in 0..sky_section_count {
        write_varint(&mut data, 2048)?;
        data.extend_from_slice(&full_light);
    }

    // Block Light Arrays
    write_varint(&mut data, 0)?;

    Ok(Bytes::from(data))
}

fn create_superflat_chunk_sections() -> Vec<u8> {
    let mut data = Vec::new();

    for section_y in 0..24 {
        let block_count: i16 = if section_y == 4 { 16 * 16 * 4 } else { 0 };
        data.extend_from_slice(&block_count.to_be_bytes());

        if section_y == 4 {
            write_superflat_section(&mut data);
        } else {
            data.push(0);
            write_varint_vec(&mut data, 0);
        }

        // Biomes
        data.push(0);
        write_varint_vec(&mut data, 0);
    }

    data
}

fn write_superflat_section(data: &mut Vec<u8>) {
    use mc_data::blocks;

    data.push(4); // 4 bits per entry

    // Palette
    write_varint_vec(data, 4);
    write_varint_vec(data, blocks::AIR.id() as i32);
    write_varint_vec(data, blocks::BEDROCK.id() as i32);
    write_varint_vec(data, blocks::DIRT.id() as i32);
    write_varint_vec(data, blocks::GRASS_BLOCK.id() as i32);

    // Data array
    for y in 0..16 {
        for _z in 0..16 {
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

fn write_varint_vec(buf: &mut Vec<u8>, value: i32) {
    write_varint(buf, value).expect("varint write");
}
