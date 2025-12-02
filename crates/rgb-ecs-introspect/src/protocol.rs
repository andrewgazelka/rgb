//! Request/response protocol for dashboard communication.
//!
//! Uses crossbeam channels for lock-free communication between the async
//! web server and the synchronous ECS world on the main thread.

use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender, bounded};
use rgb_ecs::{Component, Entity};
use serde::{Deserialize, Serialize};

use crate::IntrospectRegistry;
use crate::history::HistoryEntry;

/// Channels for dashboard communication.
pub struct IntrospectChannels {
    /// Send requests to the ECS world.
    pub request_tx: Sender<IntrospectRequest>,
    /// Receive requests in the ECS world.
    pub request_rx: Receiver<IntrospectRequest>,
}

impl IntrospectChannels {
    /// Create a new channel pair with bounded capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (request_tx, request_rx) = bounded(capacity);
        Self {
            request_tx,
            request_rx,
        }
    }

    /// Create channels with default capacity (64).
    #[must_use]
    pub fn default_capacity() -> Self {
        Self::new(64)
    }
}

impl Default for IntrospectChannels {
    fn default() -> Self {
        Self::default_capacity()
    }
}

/// Component for receiving introspection requests in the ECS world.
#[derive(Component, Clone)]
#[component(opaque)]
pub struct IntrospectIngress {
    /// Receiver for incoming requests.
    pub rx: Receiver<IntrospectRequest>,
    /// Shared registry of introspectable components.
    pub registry: Arc<IntrospectRegistry>,
}

/// Request from web server to ECS world.
pub enum IntrospectRequest {
    /// Get world-level statistics and global components.
    GetWorld {
        response: oneshot::Sender<WorldResponse>,
    },

    /// List entities, optionally filtered by component.
    ListEntities {
        filter: Option<Vec<String>>,
        limit: Option<usize>,
        offset: Option<usize>,
        response: oneshot::Sender<ListEntitiesResponse>,
    },

    /// Get a single entity with all its components.
    GetEntity {
        entity: Entity,
        response: oneshot::Sender<EntityResponse>,
    },

    /// Get a specific component from an entity.
    GetComponent {
        entity: Entity,
        component: String,
        response: oneshot::Sender<ComponentResponse>,
    },

    /// Update a component on an entity.
    UpdateComponent {
        entity: Entity,
        component: String,
        value: serde_json::Value,
        response: oneshot::Sender<UpdateResponse>,
    },

    /// Add a component to an entity.
    AddComponent {
        entity: Entity,
        component: String,
        value: serde_json::Value,
        response: oneshot::Sender<UpdateResponse>,
    },

    /// Remove a component from an entity.
    RemoveComponent {
        entity: Entity,
        component: String,
        response: oneshot::Sender<UpdateResponse>,
    },

    /// Spawn a new entity.
    SpawnEntity {
        name: Option<String>,
        components: Vec<(String, serde_json::Value)>,
        response: oneshot::Sender<SpawnResponse>,
    },

    /// Despawn an entity.
    DespawnEntity {
        entity: Entity,
        response: oneshot::Sender<UpdateResponse>,
    },

    /// Execute a query.
    Query {
        spec: QuerySpec,
        response: oneshot::Sender<QueryResponse>,
    },

    /// Get all registered component types.
    GetComponentTypes {
        response: oneshot::Sender<ComponentTypesResponse>,
    },

    /// Get chunk data for the map view.
    GetChunks {
        response: oneshot::Sender<ChunksResponse>,
    },

    /// Get component history for an entity.
    GetHistory {
        entity: Option<Entity>,
        component: Option<String>,
        limit: Option<usize>,
        response: oneshot::Sender<HistoryResponse>,
    },

    /// Revert a component to a specific history entry.
    RevertToEntry {
        entry_id: u64,
        response: oneshot::Sender<UpdateResponse>,
    },
}

/// Query specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySpec {
    /// Components to fetch (must have, return data).
    #[serde(default)]
    pub with: Vec<String>,
    /// Optional components (return if present).
    #[serde(default)]
    pub optional: Vec<String>,
    /// Filter components (must have, don't return).
    #[serde(default)]
    pub filter: Vec<String>,
    /// Exclude components (must NOT have).
    #[serde(default)]
    pub without: Vec<String>,
    /// Maximum results to return.
    pub limit: Option<usize>,
    /// Offset for pagination.
    pub offset: Option<usize>,
}

// Response types

/// World-level statistics.
#[derive(Debug, Clone, Serialize)]
pub struct WorldResponse {
    pub entity_count: u32,
    pub archetype_count: usize,
    pub component_count: usize,
    pub globals: serde_json::Value,
}

/// List of entities.
#[derive(Debug, Clone, Serialize)]
pub struct ListEntitiesResponse {
    pub entities: Vec<EntitySummary>,
    pub total: usize,
}

/// Summary of an entity (for list views).
#[derive(Debug, Clone, Serialize)]
pub struct EntitySummary {
    pub id: u64,
    pub name: Option<String>,
    pub components: Vec<String>,
}

/// Full entity details.
#[derive(Debug, Clone, Serialize)]
pub struct EntityResponse {
    pub found: bool,
    pub id: u64,
    pub name: Option<String>,
    pub components: Vec<ComponentValue>,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
}

/// Component with its value.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentValue {
    pub name: String,
    pub full_name: String,
    pub value: serde_json::Value,
    pub is_opaque: bool,
    /// Human-readable summary for opaque components (e.g., "45.2 KB")
    pub opaque_info: Option<String>,
    pub schema: Option<serde_json::Value>,
}

/// Single component response.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentResponse {
    pub found: bool,
    pub value: Option<ComponentValue>,
}

/// Result of an update operation.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// Result of spawning an entity.
#[derive(Debug, Clone, Serialize)]
pub struct SpawnResponse {
    pub success: bool,
    pub entity: Option<u64>,
    pub error: Option<String>,
}

/// Query results.
#[derive(Debug, Clone, Serialize)]
pub struct QueryResponse {
    pub entities: Vec<QueryResultRow>,
    pub total: usize,
    pub execution_time_us: u64,
}

/// Single row in query results.
#[derive(Debug, Clone, Serialize)]
pub struct QueryResultRow {
    pub entity: u64,
    pub name: Option<String>,
    pub components: serde_json::Map<String, serde_json::Value>,
}

/// Registered component types.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentTypesResponse {
    pub types: Vec<ComponentTypeInfo>,
}

/// Information about a component type.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentTypeInfo {
    pub id: u32,
    pub name: String,
    pub full_name: String,
    pub size: usize,
    pub is_opaque: bool,
    pub schema: Option<serde_json::Value>,
}

/// Chunk data for map view.
#[derive(Debug, Clone, Serialize)]
pub struct ChunksResponse {
    pub chunks: Vec<ChunkInfo>,
}

/// Information about a loaded chunk.
#[derive(Debug, Clone, Serialize)]
pub struct ChunkInfo {
    pub x: i32,
    pub z: i32,
    pub color: String, // "red", "green", "blue"
    pub loaded: bool,
}

/// Component change history response.
#[derive(Debug, Clone, Serialize)]
pub struct HistoryResponse {
    pub entries: Vec<HistoryEntry>,
    pub total: usize,
}

/// Simple oneshot channel for responses.
pub mod oneshot {
    use crossbeam_channel::bounded;

    pub struct Sender<T>(crossbeam_channel::Sender<T>);
    pub struct Receiver<T>(crossbeam_channel::Receiver<T>);

    impl<T> Sender<T> {
        pub fn send(self, value: T) -> Result<(), T> {
            self.0.send(value).map_err(|e| e.0)
        }
    }

    impl<T> Receiver<T> {
        pub fn recv(self) -> Result<T, RecvError> {
            self.0.recv().map_err(|_| RecvError)
        }

        pub fn recv_timeout(self, timeout: std::time::Duration) -> Result<T, RecvTimeoutError> {
            self.0.recv_timeout(timeout).map_err(|e| match e {
                crossbeam_channel::RecvTimeoutError::Timeout => RecvTimeoutError::Timeout,
                crossbeam_channel::RecvTimeoutError::Disconnected => RecvTimeoutError::Disconnected,
            })
        }
    }

    #[derive(Debug)]
    pub struct RecvError;

    #[derive(Debug)]
    pub enum RecvTimeoutError {
        Timeout,
        Disconnected,
    }

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = bounded(1);
        (Sender(tx), Receiver(rx))
    }
}
