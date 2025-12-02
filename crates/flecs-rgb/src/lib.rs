//! RGB Spatial Partitioning for Flecs ECS
//!
//! This crate provides a 3-coloring system for lock-free parallel execution of spatial regions.
//! Red, Green, and Blue regions can be processed in parallel within their color phase.
//!
//! # Architecture
//!
//! ```text
//! Region (color: Red|Green|Blue)
//!   └── Chunk (ChildOf Region)
//!         └── Entity (position determines which chunk)
//! ```
//!
//! # Tick Execution Model
//!
//! ```text
//! Tick N:
//! 1. Global phase (sequential) - network ingress, etc.
//! 2. Red regions in parallel (rayon) + readonly_end() to merge
//! 3. Green regions in parallel (rayon) + readonly_end() to merge
//! 4. Blue regions in parallel (rayon) + readonly_end() to merge
//! 5. Global phase (sequential) - network egress, etc.
//! ```

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]

mod event;
mod region;
mod scoped;
mod tick;

pub use event::{Event, EventHandler, EventWorldExt, HandlerInfo};
pub use region::{chebyshev_distance, Chunk, Position, Region, RegionColor};
pub use scoped::{ScopeError, ScopedWorld};
pub use tick::{RgbScheduler, TickPhase};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        chebyshev_distance, Chunk, Event, EventHandler, EventWorldExt, HandlerInfo, Position,
        Region, RegionColor, RgbScheduler, ScopeError, ScopedWorld,
    };
}
