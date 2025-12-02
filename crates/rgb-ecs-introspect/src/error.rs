//! Error types for introspection operations.

use thiserror::Error;

/// Errors that can occur during introspection operations.
#[derive(Debug, Error)]
pub enum IntrospectError {
    /// JSON serialization/deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Entity was not found in the world.
    #[error("Entity not found: {0}")]
    EntityNotFound(u64),

    /// Component type was not found in the registry.
    #[error("Component type not found: {0}")]
    ComponentNotFound(String),

    /// Component is opaque and cannot be serialized.
    #[error("Component is opaque: {0}")]
    OpaqueComponent(String),

    /// Deserialization failed for a component.
    #[error("Deserialization failed for {component}: {error}")]
    DeserializationFailed { component: String, error: String },

    /// Component type is not introspectable (not registered).
    #[error("Component not introspectable: {0}")]
    NotIntrospectable(String),

    /// Request timed out waiting for response.
    #[error("Request timeout")]
    Timeout,

    /// Channel was disconnected.
    #[error("Channel disconnected")]
    ChannelDisconnected,

    /// Invalid entity ID format.
    #[error("Invalid entity ID: {0}")]
    InvalidEntityId(String),
}
