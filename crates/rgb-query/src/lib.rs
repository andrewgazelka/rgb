//! Query system and Scope for RGB ECS.
//!
//! A `Scope` provides access to a subset of the world (typically a 3x3 chunk neighborhood)
//! with queries that are implicitly filtered to only access entities in that scope.
//!
//! # Design
//!
//! During RGB parallel execution:
//! - Each chunk gets exclusive access to its 3x3 neighborhood
//! - A `Scope` represents this neighborhood
//! - All queries through a scope are implicitly filtered to the accessible chunks
//!
//! # Example
//!
//! ```ignore
//! #[rpc]
//! fn on_tick(ctx: &mut RpcContext) {
//!     let scope = ctx.scope();
//!
//!     // Query returns owned values
//!     for (entity, health) in scope.query::<Health>().iter() {
//!         if health.value < 10 {
//!             scope.insert(entity, Poisoned);
//!         }
//!     }
//! }
//! ```

mod scope;

pub use scope::{ChunkId, Neighborhood, Scope};
