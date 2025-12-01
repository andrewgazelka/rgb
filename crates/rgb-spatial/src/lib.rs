//! RGB Spatial Partitioning
//!
//! Divides the world into cells with RGB 3-coloring for parallel execution.
//! Same-colored cells never share edges, enabling lock-free parallel processing.

pub mod cell;
pub mod grid;

pub use cell::{Cell, CellId, Color};
pub use grid::SpatialGrid;
