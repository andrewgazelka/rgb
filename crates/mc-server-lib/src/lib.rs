//! Minecraft server ECS library - re-exports from module crates
//!
//! This crate is a compatibility shim for the old mc-server binary.
//! New code should use the module crates directly.

pub use flecs_ecs::prelude::*;

// Re-export network types for mc-server compatibility
pub use module_network_components::{
    DisconnectEvent, IncomingPacket, NetworkChannels, OutgoingPacket,
};

/// Configuration for the Minecraft server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port for the Flecs REST API explorer (default: 27750)
    pub rest_port: u16,
    /// Enable stats collection for the explorer
    pub enable_stats: bool,
    /// Target frames per second for the game loop
    pub target_fps: f32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            rest_port: 27750,
            enable_stats: true,
            target_fps: 20.0, // Minecraft runs at 20 TPS
        }
    }
}
