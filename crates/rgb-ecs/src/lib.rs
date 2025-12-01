// Allow unsafe code in ECS - necessary for low-level component storage
#![allow(unsafe_code)]
// Allow missing docs for now
#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::float_cmp)]

//! RGB ECS - Archetype-based Entity Component System
//!
//! Designed for tick-based parallel execution with RGB spatial partitioning.
//!
//! # Key Concepts
//!
//! - **Entity**: A unique identifier for a game object
//! - **Component**: Simple, flat data attached to entities (e.g., Position, Velocity)
//! - **Archetype**: A unique combination of component types
//! - **Relation/Pair**: A relationship between entities `(Relation, Target)`
//! - **Global**: Marker for global entities (read-only in parallel, writable in sequential)
//!
//! # Component Design
//!
//! Components must be simple, flat data types. Use `#[derive(Component)]`:
//!
//! ```ignore
//! use rgb_ecs::Component;
//!
//! // GOOD: Simple flat data
//! #[derive(Component, Clone)]
//! struct Position { x: f64, y: f64, z: f64 }
//!
//! // BAD: Collections are forbidden - use relations instead!
//! #[derive(Component, Clone)]
//! struct Inventory { items: Vec<Item> }  // Compile error!
//!
//! // GOOD: The relational way
//! #[derive(Component, Clone)]
//! struct Item { id: u32, count: u8, slot: u8 }
//! // Spawn items as: world.spawn((Item { ... }, Pair::<ChildOf>(player)))
//! ```
//!
//! # Access Patterns
//!
//! All component access returns owned values (SpacetimeDB pattern):
//! - `get<T>()` - Returns owned `T` (requires `Clone`)
//! - `update<T>()` - Write back modified value
//! - `insert<T>()` - Add new component
//! - `remove<T>()` - Remove and return component
//!
//! # Global State
//!
//! Use `Entity::WORLD` for global state instead of singletons:
//! ```ignore
//! // Store global config on the WORLD entity
//! world.insert(Entity::WORLD, GameConfig { tick_rate: 20 });
//!
//! // Read it (works in parallel and sequential)
//! let config = world.get::<GameConfig>(Entity::WORLD)?;
//!
//! // Update it (only in sequential context)
//! world.update(Entity::WORLD, new_config);
//! ```

mod archetype;
mod component;
mod entity;
mod query;
mod relation;
mod storage;
mod world;

pub use archetype::{Archetype, ArchetypeId};
pub use component::{Component, ComponentId, ComponentInfo, ComponentRegistry};
pub use entity::{Entity, EntityId, Generation};
pub use query::{Query, QueryBuilder, QueryIter, QueryRow, QueryTerm, TermAccess};
pub use relation::{ChildOf, ContainedIn, InstanceOf, OwnedBy, Pair, PairId, Requires};
pub use storage::{Column, ComponentStorage};
pub use world::{Global, Plugin, World};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        ChildOf, Component, ContainedIn, Entity, Global, InstanceOf, OwnedBy, Pair, Plugin, Query,
        QueryBuilder, QueryRow, Requires, World,
    };
}
