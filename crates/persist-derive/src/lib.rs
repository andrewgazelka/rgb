//! Derive macros for the persist crate.
//!
//! This crate is currently a placeholder. The persistence system now uses
//! the `PersistExt` trait which doesn't require a derive macro.
//!
//! Usage:
//! ```ignore
//! use persist::PersistExt;
//!
//! // Register component for persistence
//! world.component::<Position>().persist::<Uuid>();
//! ```

// No derive macros needed for the new design.
// The crate is kept for potential future use.
