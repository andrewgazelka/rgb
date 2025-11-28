mod chunk_manager;
mod color;
mod components;
mod pos;

pub use chunk_manager::{
    get_neighbor, link_chunk_neighbors, spawn_chunk, unlink_chunk_neighbors, ChunkIndex,
};
pub use color::Color;
pub use components::{
    Active, CellData, Dirty, Direction, NeighborE, NeighborN, NeighborNE, NeighborNW, NeighborS,
    NeighborSE, NeighborSW, NeighborW, NextCellData, SimColor,
};
pub use pos::{CellPos, ChunkPos, RegionPos, CHUNK_SIZE, REGION_SIZE};

// Re-export flecs for convenience
pub use flecs_ecs;
