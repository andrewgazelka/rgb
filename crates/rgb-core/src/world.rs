use std::collections::{HashMap, HashSet};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::chunk::Chunk;
use crate::color::Color;
use crate::pos::{ChunkPos, RegionPos};
use crate::simulation::Simulation;
use crate::slice::WorldSlice;

pub struct World<S: Simulation> {
    chunks: HashMap<ChunkPos, S::Chunk>,
    simulation: S,
}

impl<S: Simulation> World<S> {
    pub fn new(simulation: S) -> Self {
        Self {
            chunks: HashMap::new(),
            simulation,
        }
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&S::Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> &mut S::Chunk {
        self.chunks.entry(pos).or_default()
    }

    pub fn set_chunk(&mut self, pos: ChunkPos, chunk: S::Chunk) {
        if chunk.is_empty() {
            self.chunks.remove(&pos);
        } else {
            self.chunks.insert(pos, chunk);
        }
    }

    pub fn chunks(&self) -> impl Iterator<Item = (ChunkPos, &S::Chunk)> {
        self.chunks.iter().map(|(&pos, chunk)| (pos, chunk))
    }

    fn active_regions(&self) -> HashSet<RegionPos> {
        self.chunks
            .keys()
            .map(|&pos| pos.containing_region())
            .collect()
    }

    fn regions_by_color(&self, color: Color) -> Vec<RegionPos> {
        self.active_regions()
            .into_iter()
            .filter(|&pos| Color::from_region(pos) == color)
            .collect()
    }

    fn extract_slice(&self, region: RegionPos) -> WorldSlice<S::Chunk> {
        let mut slice = WorldSlice::new(region);
        let positions: Vec<_> = slice.surrounding_chunk_positions().collect();

        for pos in positions {
            if let Some(chunk) = self.chunks.get(&pos) {
                slice.set(pos, Some(chunk.clone()));
            }
        }

        slice
    }

    fn merge_slice(&mut self, slice: WorldSlice<S::Chunk>) {
        for pos in slice.center_chunks() {
            match slice.get(pos) {
                Some(chunk) if !chunk.is_empty() => {
                    self.chunks.insert(pos, chunk.clone());
                }
                _ => {
                    self.chunks.remove(&pos);
                }
            }
        }
    }

    pub fn step(&mut self) {
        for color in Color::ALL {
            let regions = self.regions_by_color(color);

            if regions.is_empty() {
                continue;
            }

            let slices: Vec<_> = regions
                .iter()
                .map(|&region| self.extract_slice(region))
                .collect();

            let results: Vec<_> = slices
                .into_par_iter()
                .map(|mut slice| {
                    self.simulation.step_region(&mut slice);
                    slice
                })
                .collect();

            for slice in results {
                self.merge_slice(slice);
            }
        }
    }

    pub fn simulation(&self) -> &S {
        &self.simulation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slice::WorldSlice;

    #[derive(Clone, Default)]
    struct CountChunk(u32);

    impl Chunk for CountChunk {
        fn is_empty(&self) -> bool {
            self.0 == 0
        }
    }

    struct IncrementSim;

    impl Simulation for IncrementSim {
        type Chunk = CountChunk;

        fn step_region(&self, slice: &mut WorldSlice<Self::Chunk>) {
            for pos in slice.center_chunks().collect::<Vec<_>>() {
                if let Some(chunk) = slice.get_mut(pos) {
                    chunk.0 += 1;
                }
            }
        }
    }

    #[test]
    fn world_step() {
        let mut world = World::new(IncrementSim);

        world.set_chunk(ChunkPos::new(0, 0), CountChunk(1));
        world.set_chunk(ChunkPos::new(4, 0), CountChunk(1));

        world.step();

        assert_eq!(world.get_chunk(ChunkPos::new(0, 0)).map(|c| c.0), Some(2));
        assert_eq!(world.get_chunk(ChunkPos::new(4, 0)).map(|c| c.0), Some(2));
    }
}
