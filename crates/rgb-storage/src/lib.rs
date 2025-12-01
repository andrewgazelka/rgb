//! Versioned storage with time-travel for RGB database.
//!
//! This crate provides persistent storage for the ECS world using Nebari's
//! versioned append-only B+tree. Every mutation is logged, enabling:
//!
//! - **Time-travel queries**: Read the world state at any historical tick
//! - **Instant revert**: Jump back to any tick without replay
//! - **Efficient storage**: Structural sharing means only changed data is written
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │  VersionedWorld                                                     │
//! │    - Wraps ECS World + Nebari storage                               │
//! │    - Batches mutations until tick commit                            │
//! │    - Each tick = one Nebari transaction                             │
//! └─────────────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │  Nebari Versioned B+Tree                                            │
//! │    - Key: (EntityId, ComponentId) → 12 bytes                        │
//! │    - Value: serialized component data                               │
//! │    - Each write gets unique sequence ID                             │
//! │    - Can scan sequences to see all history                          │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use rgb_storage::VersionedWorld;
//!
//! // Create a new versioned world
//! let mut world = VersionedWorld::create("game.db")?;
//!
//! // Spawn and modify entities (buffered in memory)
//! let player = world.spawn(Position { x: 0.0, y: 64.0, z: 0.0 });
//! world.insert(player, Health::new(20));
//!
//! // Commit tick (writes to disk atomically)
//! let tick = world.commit_tick()?;
//!
//! // Later: revert to any tick
//! world.revert_to_tick(tick)?;
//!
//! // Or read state at any tick without reverting
//! let old_health = world.get_at_tick::<Health>(player, tick)?;
//! ```

mod buffer;
mod error;
mod keys;
mod versioned_world;

pub use buffer::{Mutation, MutationBuffers};
pub use error::{StorageError, StorageResult};
pub use keys::ComponentKey;
pub use versioned_world::VersionedWorld;

/// A tick identifier (monotonically increasing).
pub type TickId = u64;
