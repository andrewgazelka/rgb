use flecs_ecs::prelude::*;

use crate::pos::CHUNK_SIZE;

/// Cell data for a chunk (16x16 grid of bools)
#[derive(Component, Clone)]
pub struct CellData {
    pub cells: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Default for CellData {
    fn default() -> Self {
        Self {
            cells: [[false; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }
}

impl CellData {
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.cells[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, alive: bool) {
        self.cells[y][x] = alive;
    }

    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(|&c| !c))
    }

    pub fn count_alive(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c)
            .count()
    }
}

/// Double-buffer for safe updates during simulation
#[derive(Component, Clone)]
pub struct NextCellData {
    pub cells: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Default for NextCellData {
    fn default() -> Self {
        Self {
            cells: [[false; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }
}

/// Parallel processing color (0-8) based on region position
#[derive(Component, Copy, Clone)]
#[flecs(meta)]
pub struct SimColor(pub u8);

/// Tag: chunk needs texture update
#[derive(Component, Default)]
pub struct Dirty;

/// Tag: chunk has live cells
#[derive(Component, Default)]
pub struct Active;

// ============================================
// Neighbor Relationship Components (8 directions)
// ============================================

/// North neighbor (+y direction)
#[derive(Component)]
pub struct NeighborN;

/// South neighbor (-y direction)
#[derive(Component)]
pub struct NeighborS;

/// East neighbor (+x direction)
#[derive(Component)]
pub struct NeighborE;

/// West neighbor (-x direction)
#[derive(Component)]
pub struct NeighborW;

/// Northeast neighbor
#[derive(Component)]
pub struct NeighborNE;

/// Northwest neighbor
#[derive(Component)]
pub struct NeighborNW;

/// Southeast neighbor
#[derive(Component)]
pub struct NeighborSE;

/// Southwest neighbor
#[derive(Component)]
pub struct NeighborSW;

/// Direction enum for neighbor lookups
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Component)]
#[repr(C)]
#[flecs(meta)]
pub enum Direction {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::N,
        Direction::S,
        Direction::E,
        Direction::W,
        Direction::NE,
        Direction::NW,
        Direction::SE,
        Direction::SW,
    ];

    /// Returns (dx, dy) offset for this direction
    pub const fn offset(self) -> (i32, i32) {
        match self {
            Direction::N => (0, 1),
            Direction::S => (0, -1),
            Direction::E => (1, 0),
            Direction::W => (-1, 0),
            Direction::NE => (1, 1),
            Direction::NW => (-1, 1),
            Direction::SE => (1, -1),
            Direction::SW => (-1, -1),
        }
    }

    /// Returns the opposite direction
    pub const fn opposite(self) -> Direction {
        match self {
            Direction::N => Direction::S,
            Direction::S => Direction::N,
            Direction::E => Direction::W,
            Direction::W => Direction::E,
            Direction::NE => Direction::SW,
            Direction::NW => Direction::SE,
            Direction::SE => Direction::NW,
            Direction::SW => Direction::NE,
        }
    }
}
