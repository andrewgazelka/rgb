//! Packet handler components for relation-based dispatch
//!
//! Design:
//! - Each handler is an entity with `PacketHandler`, `Priority`, and `HandlerFor` components
//! - `HandlerFor { state, packet_id }` specifies which packet the handler processes
//! - Multiple handlers can have the same `HandlerFor` (composable)
//! - Dispatch queries all handlers matching the incoming packet

use flecs_ecs::prelude::*;

use super::ConnectionState;

/// Handler function signature
/// Takes the connection entity and raw packet data
pub type HandlerFn = fn(EntityView<'_>, &[u8]);

/// Component storing the handler function
#[derive(Component)]
pub struct PacketHandler {
    pub handler: HandlerFn,
}

/// Priority for handler execution order (lower = runs first)
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Priority(pub i32);

/// Specifies which packet type this handler processes
#[derive(Component, Debug, Clone, Copy)]
pub struct HandlerFor {
    pub state: ConnectionState,
    pub packet_id: i32,
}
