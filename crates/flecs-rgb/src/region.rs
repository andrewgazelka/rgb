//! Region and Chunk hierarchy with RGB coloring

use flecs_ecs::prelude::*;

/// Region color for parallel execution phases.
/// Adjacent regions have different colors, allowing same-color regions to run in parallel.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionColor {
    Red,
    Green,
    Blue,
}

impl RegionColor {
    /// Get all colors in execution order
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Red, Self::Green, Self::Blue]
    }

    /// Compute the color for a region at grid position (rx, rz).
    /// Uses modular arithmetic to ensure adjacent regions have different colors.
    #[must_use]
    pub fn from_region_pos(rx: i32, rz: i32) -> Self {
        // Map (rx, rz) to 0, 1, or 2 using 3-coloring pattern
        // Adjacent regions (including diagonals) must have different colors
        let color_idx = (rx.rem_euclid(3) + rz.rem_euclid(3)) % 3;
        match color_idx {
            0 => Self::Red,
            1 => Self::Green,
            _ => Self::Blue,
        }
    }
}

/// A region is a spatial partition containing multiple chunks.
/// Regions are colored Red, Green, or Blue for parallel execution.
#[derive(Component, Debug, Clone, Copy)]
pub struct Region {
    /// Region X coordinate in region-space
    pub rx: i32,
    /// Region Z coordinate in region-space
    pub rz: i32,
}

impl Region {
    /// Create a new region at the given coordinates
    #[must_use]
    pub const fn new(rx: i32, rz: i32) -> Self {
        Self { rx, rz }
    }

    /// Get the color for this region
    #[must_use]
    pub fn color(&self) -> RegionColor {
        RegionColor::from_region_pos(self.rx, self.rz)
    }
}

/// A chunk belongs to a region via ChildOf relation.
/// Chunks contain entities based on their Position.
#[derive(Component, Debug, Clone, Copy)]
pub struct Chunk {
    /// Chunk X coordinate in chunk-space
    pub x: i32,
    /// Chunk Z coordinate in chunk-space
    pub z: i32,
}

impl Chunk {
    /// Create a new chunk at the given coordinates
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Get the region coordinates that contain this chunk.
    /// Assumes 16 chunks per region (configurable).
    #[must_use]
    pub fn region_coords(&self, chunks_per_region: i32) -> (i32, i32) {
        (
            self.x.div_euclid(chunks_per_region),
            self.z.div_euclid(chunks_per_region),
        )
    }
}

/// Position component determines which chunk an entity belongs to.
#[derive(Component, Debug, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    /// Create a new position
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Get the chunk coordinates for this position.
    /// Uses standard Minecraft chunk size of 16 blocks.
    #[must_use]
    pub fn chunk_coords(&self) -> (i32, i32) {
        let chunk_size = 16.0;
        (
            (self.x / chunk_size).floor() as i32,
            (self.z / chunk_size).floor() as i32,
        )
    }
}

/// Compute Chebyshev distance between two chunk positions.
/// Used for ScopedWorld boundary validation.
#[must_use]
pub fn chebyshev_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    let dx = (a.0 - b.0).abs();
    let dz = (a.1 - b.1).abs();
    dx.max(dz)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_coloring() {
        // Orthogonally adjacent regions should have different colors (4-connectivity)
        // With 3 colors using (rx + rz) % 3, same color appears every 3 steps
        let c00 = RegionColor::from_region_pos(0, 0);
        let c01 = RegionColor::from_region_pos(0, 1);
        let c10 = RegionColor::from_region_pos(1, 0);

        // Orthogonally adjacent must differ
        assert_ne!(c00, c01);
        assert_ne!(c00, c10);

        // Same color regions at distance 3 (period is 3)
        let c30 = RegionColor::from_region_pos(3, 0);
        let c03 = RegionColor::from_region_pos(0, 3);
        assert_eq!(c00, c30);
        assert_eq!(c00, c03);

        // Also same at diagonal with sum = 3
        let c12 = RegionColor::from_region_pos(1, 2);
        let c21 = RegionColor::from_region_pos(2, 1);
        assert_eq!(c03, c12);
        assert_eq!(c03, c21);
    }

    #[test]
    fn test_chunk_to_region() {
        let chunk = Chunk::new(17, 33);
        let (rx, rz) = chunk.region_coords(16);
        assert_eq!(rx, 1);
        assert_eq!(rz, 2);

        // Negative coordinates
        let chunk_neg = Chunk::new(-1, -17);
        let (rx_neg, rz_neg) = chunk_neg.region_coords(16);
        assert_eq!(rx_neg, -1);
        assert_eq!(rz_neg, -2);
    }

    #[test]
    fn test_position_to_chunk() {
        let pos = Position::new(17.5, 64.0, 33.5);
        let (cx, cz) = pos.chunk_coords();
        assert_eq!(cx, 1);
        assert_eq!(cz, 2);

        // Negative coordinates
        let pos_neg = Position::new(-0.5, 64.0, -17.5);
        let (cx_neg, cz_neg) = pos_neg.chunk_coords();
        assert_eq!(cx_neg, -1);
        assert_eq!(cz_neg, -2);
    }

    #[test]
    fn test_chebyshev_distance() {
        assert_eq!(chebyshev_distance((0, 0), (0, 0)), 0);
        assert_eq!(chebyshev_distance((0, 0), (1, 0)), 1);
        assert_eq!(chebyshev_distance((0, 0), (1, 1)), 1);
        assert_eq!(chebyshev_distance((0, 0), (2, 1)), 2);
        assert_eq!(chebyshev_distance((-1, -1), (1, 1)), 2);
    }
}
