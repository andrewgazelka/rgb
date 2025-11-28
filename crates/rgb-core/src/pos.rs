pub const CHUNK_SIZE: usize = 16;
pub const REGION_SIZE: usize = 4;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct RegionPos {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct CellPos {
    pub x: i64,
    pub y: i64,
}

impl ChunkPos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn containing_region(self) -> RegionPos {
        RegionPos {
            x: self.x.div_euclid(REGION_SIZE as i32),
            y: self.y.div_euclid(REGION_SIZE as i32),
        }
    }

    pub const fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl RegionPos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn chunks(self) -> impl Iterator<Item = ChunkPos> {
        let base_x = self.x * REGION_SIZE as i32;
        let base_y = self.y * REGION_SIZE as i32;
        (0..REGION_SIZE as i32).flat_map(move |dy| {
            (0..REGION_SIZE as i32).map(move |dx| ChunkPos::new(base_x + dx, base_y + dy))
        })
    }

    pub const fn top_left_chunk(self) -> ChunkPos {
        ChunkPos {
            x: self.x * REGION_SIZE as i32,
            y: self.y * REGION_SIZE as i32,
        }
    }
}

impl CellPos {
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub const fn containing_chunk(self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(CHUNK_SIZE as i64) as i32,
            y: self.y.div_euclid(CHUNK_SIZE as i64) as i32,
        }
    }

    pub const fn local_in_chunk(self) -> (usize, usize) {
        (
            self.x.rem_euclid(CHUNK_SIZE as i64) as usize,
            self.y.rem_euclid(CHUNK_SIZE as i64) as usize,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_to_region() {
        assert_eq!(ChunkPos::new(0, 0).containing_region(), RegionPos::new(0, 0));
        assert_eq!(ChunkPos::new(3, 3).containing_region(), RegionPos::new(0, 0));
        assert_eq!(ChunkPos::new(4, 0).containing_region(), RegionPos::new(1, 0));
        assert_eq!(
            ChunkPos::new(-1, -1).containing_region(),
            RegionPos::new(-1, -1)
        );
    }

    #[test]
    fn region_chunks() {
        let chunks: Vec<_> = RegionPos::new(0, 0).chunks().collect();
        assert_eq!(chunks.len(), 16);
        assert!(chunks.contains(&ChunkPos::new(0, 0)));
        assert!(chunks.contains(&ChunkPos::new(3, 3)));
    }

    #[test]
    fn cell_to_chunk() {
        assert_eq!(CellPos::new(0, 0).containing_chunk(), ChunkPos::new(0, 0));
        assert_eq!(CellPos::new(15, 15).containing_chunk(), ChunkPos::new(0, 0));
        assert_eq!(CellPos::new(16, 0).containing_chunk(), ChunkPos::new(1, 0));
        assert_eq!(
            CellPos::new(-1, -1).containing_chunk(),
            ChunkPos::new(-1, -1)
        );
    }
}
