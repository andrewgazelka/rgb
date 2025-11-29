mod chunk_manager;
mod color;
mod components;
mod pos;

pub use chunk_manager::{
    ChunkIndex, get_neighbor, link_chunk_neighbors, spawn_chunk, unlink_chunk_neighbors,
};
pub use color::Color;
pub use components::{
    Active, CellData, Direction, Dirty, NeighborE, NeighborN, NeighborNE, NeighborNW, NeighborS,
    NeighborSE, NeighborSW, NeighborW, NextCellData, SimColor,
};
pub use pos::{CHUNK_SIZE, CellPos, ChunkPos, REGION_SIZE, RegionPos};

// Re-export flecs for convenience
pub use flecs_ecs;
