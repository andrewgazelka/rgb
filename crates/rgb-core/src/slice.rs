use crate::chunk::Chunk;
use crate::pos::{ChunkPos, RegionPos, REGION_SIZE};

pub const SLICE_SIZE: usize = REGION_SIZE + 2;

pub struct WorldSlice<C> {
    pub center: RegionPos,
    pub chunks: [[Option<C>; SLICE_SIZE]; SLICE_SIZE],
}

impl<C: Chunk> WorldSlice<C> {
    pub fn new(center: RegionPos) -> Self {
        Self {
            center,
            chunks: std::array::from_fn(|_| std::array::from_fn(|_| None)),
        }
    }

    fn local_index(&self, chunk: ChunkPos) -> Option<(usize, usize)> {
        let base = self.center.top_left_chunk();
        let dx = chunk.x - base.x + 1;
        let dy = chunk.y - base.y + 1;

        if dx >= 0 && dx < SLICE_SIZE as i32 && dy >= 0 && dy < SLICE_SIZE as i32 {
            Some((dx as usize, dy as usize))
        } else {
            None
        }
    }

    pub fn get(&self, chunk: ChunkPos) -> Option<&C> {
        let (x, y) = self.local_index(chunk)?;
        self.chunks[y][x].as_ref()
    }

    pub fn get_mut(&mut self, chunk: ChunkPos) -> Option<&mut C> {
        let (x, y) = self.local_index(chunk)?;
        self.chunks[y][x].as_mut()
    }

    pub fn set(&mut self, chunk: ChunkPos, data: Option<C>) {
        if let Some((x, y)) = self.local_index(chunk) {
            self.chunks[y][x] = data;
        }
    }

    pub fn center_chunks(&self) -> impl Iterator<Item = ChunkPos> {
        self.center.chunks()
    }

    pub fn is_center(&self, chunk: ChunkPos) -> bool {
        let base = self.center.top_left_chunk();
        chunk.x >= base.x
            && chunk.x < base.x + REGION_SIZE as i32
            && chunk.y >= base.y
            && chunk.y < base.y + REGION_SIZE as i32
    }

    pub fn surrounding_chunk_positions(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        let base = self.center.top_left_chunk();
        (0..SLICE_SIZE as i32).flat_map(move |dy| {
            (0..SLICE_SIZE as i32).map(move |dx| ChunkPos::new(base.x - 1 + dx, base.y - 1 + dy))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct TestChunk(u32);

    impl Chunk for TestChunk {
        fn is_empty(&self) -> bool {
            self.0 == 0
        }
    }

    #[test]
    fn slice_indexing() {
        let mut slice: WorldSlice<TestChunk> = WorldSlice::new(RegionPos::new(0, 0));

        slice.set(ChunkPos::new(0, 0), Some(TestChunk(42)));
        assert_eq!(slice.get(ChunkPos::new(0, 0)).map(|c| c.0), Some(42));

        slice.set(ChunkPos::new(-1, -1), Some(TestChunk(1)));
        assert_eq!(slice.get(ChunkPos::new(-1, -1)).map(|c| c.0), Some(1));

        assert!(slice.get(ChunkPos::new(-2, -2)).is_none());
    }

    #[test]
    fn center_detection() {
        let slice: WorldSlice<TestChunk> = WorldSlice::new(RegionPos::new(0, 0));

        assert!(slice.is_center(ChunkPos::new(0, 0)));
        assert!(slice.is_center(ChunkPos::new(3, 3)));
        assert!(!slice.is_center(ChunkPos::new(-1, 0)));
        assert!(!slice.is_center(ChunkPos::new(4, 0)));
    }
}
