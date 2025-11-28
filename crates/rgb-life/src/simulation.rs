use rgb_core::{ChunkPos, Simulation, WorldSlice, CHUNK_SIZE};

use crate::chunk::LifeChunk;

pub struct LifeSimulation;

impl LifeSimulation {
    fn get_cell(slice: &WorldSlice<LifeChunk>, chunk: ChunkPos, x: i32, y: i32) -> bool {
        let (chunk_pos, local_x, local_y) = normalize_coords(chunk, x, y);

        slice
            .get(chunk_pos)
            .map(|c| c.get(local_x, local_y))
            .unwrap_or(false)
    }

    fn count_neighbors(slice: &WorldSlice<LifeChunk>, chunk: ChunkPos, x: i32, y: i32) -> u8 {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if Self::get_cell(slice, chunk, x + dx, y + dy) {
                    count += 1;
                }
            }
        }
        count
    }
}

fn normalize_coords(chunk: ChunkPos, x: i32, y: i32) -> (ChunkPos, usize, usize) {
    let global_x = chunk.x as i64 * CHUNK_SIZE as i64 + x as i64;
    let global_y = chunk.y as i64 * CHUNK_SIZE as i64 + y as i64;

    let new_chunk = ChunkPos::new(
        global_x.div_euclid(CHUNK_SIZE as i64) as i32,
        global_y.div_euclid(CHUNK_SIZE as i64) as i32,
    );

    let local_x = global_x.rem_euclid(CHUNK_SIZE as i64) as usize;
    let local_y = global_y.rem_euclid(CHUNK_SIZE as i64) as usize;

    (new_chunk, local_x, local_y)
}

impl Simulation for LifeSimulation {
    type Chunk = LifeChunk;

    fn step_region(&self, slice: &mut WorldSlice<Self::Chunk>) {
        let mut new_chunks: Vec<(ChunkPos, LifeChunk)> = Vec::new();

        for chunk_pos in slice.center_chunks() {
            let mut new_chunk = LifeChunk::default();

            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let alive = Self::get_cell(slice, chunk_pos, x as i32, y as i32);
                    let neighbors = Self::count_neighbors(slice, chunk_pos, x as i32, y as i32);

                    let next_alive = match (alive, neighbors) {
                        (true, 2) | (true, 3) => true,
                        (false, 3) => true,
                        _ => false,
                    };

                    new_chunk.set(x, y, next_alive);
                }
            }

            new_chunks.push((chunk_pos, new_chunk));
        }

        for (pos, chunk) in new_chunks {
            slice.set(pos, Some(chunk));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rgb_core::World;

    #[test]
    fn blinker() {
        let mut world = World::new(LifeSimulation);

        {
            let chunk = world.get_chunk_mut(ChunkPos::new(0, 0));
            chunk.set(7, 7, true);
            chunk.set(8, 7, true);
            chunk.set(9, 7, true);
        }

        world.step();

        let chunk = world.get_chunk(ChunkPos::new(0, 0)).unwrap();
        assert!(!chunk.get(7, 7));
        assert!(chunk.get(8, 6));
        assert!(chunk.get(8, 7));
        assert!(chunk.get(8, 8));
        assert!(!chunk.get(9, 7));
    }

    #[test]
    fn block_stable() {
        let mut world = World::new(LifeSimulation);

        {
            let chunk = world.get_chunk_mut(ChunkPos::new(0, 0));
            chunk.set(0, 0, true);
            chunk.set(1, 0, true);
            chunk.set(0, 1, true);
            chunk.set(1, 1, true);
        }

        world.step();

        let chunk = world.get_chunk(ChunkPos::new(0, 0)).unwrap();
        assert!(chunk.get(0, 0));
        assert!(chunk.get(1, 0));
        assert!(chunk.get(0, 1));
        assert!(chunk.get(1, 1));
    }
}
