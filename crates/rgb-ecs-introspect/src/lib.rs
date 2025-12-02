//! Introspection layer for RGB ECS dashboard.
//!
//! This crate provides the `Introspectable` trait for components that can be
//! serialized to/from JSON for the web dashboard, plus the channel-based
//! protocol for safe world access from async web handlers.
//!
//! # Usage
//!
//! ```ignore
//! use rgb_ecs::Component;
//! use rgb_ecs_introspect::Introspectable;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Component, Clone, Serialize, Deserialize, Introspectable)]
//! pub struct Position { x: f64, y: f64, z: f64 }
//! ```

#![allow(unsafe_code)]
#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod error;
pub mod history;
pub mod protocol;
mod registry;
mod traits;

pub use error::IntrospectError;
pub use history::{ChangeSource, HistoryEntry, HistoryStore};
pub use protocol::{
    ChunksResponse, ComponentResponse, ComponentTypesResponse, EntityResponse, HistoryResponse,
    IntrospectChannels, IntrospectIngress, IntrospectRequest, ListEntitiesResponse, QueryResponse,
    QuerySpec, SpawnResponse, UpdateResponse, WorldResponse,
};
pub use registry::{AlignedBuffer, IntrospectInfo, IntrospectRegistry};
pub use rgb_ecs_introspect_derive::Introspectable;
pub use traits::Introspectable;
