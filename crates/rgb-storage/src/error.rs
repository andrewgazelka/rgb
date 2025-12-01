//! Storage error types.

use thiserror::Error;

/// Storage error type.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Nebari database error.
    #[error("database error: {0}")]
    Database(#[from] nebari::Error),

    /// Entity not found.
    #[error("entity not found: {0:?}")]
    EntityNotFound(rgb_ecs::Entity),

    /// Component not found.
    #[error("component not found")]
    ComponentNotFound,

    /// Invalid tick (too far in the future or past).
    #[error("invalid tick: {0}")]
    InvalidTick(crate::TickId),

    /// IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Endianness mismatch - database was created on a different architecture.
    #[error("endianness mismatch: database requires little-endian")]
    EndiannessMismatch,
}

/// Result type for storage operations.
pub type StorageResult<T> = Result<T, StorageError>;
