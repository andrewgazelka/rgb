use rgb_core::{Chunk, CHUNK_SIZE};

#[derive(Clone)]
pub struct LifeChunk {
    pub cells: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Default for LifeChunk {
    fn default() -> Self {
        Self {
            cells: [[false; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }
}

impl Chunk for LifeChunk {
    fn is_empty(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(|&c| !c))
    }
}

impl LifeChunk {
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.cells[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, alive: bool) {
        self.cells[y][x] = alive;
    }

    pub fn count_alive(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c)
            .count()
    }
}
