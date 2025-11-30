//! Network components module - connection management and packet types
//!
//! This module provides:
//! - `NetworkIngress` / `NetworkEgress` - channel endpoints for async I/O
//! - `ConnectionIndex` - maps connection IDs to ECS entities
//! - `PacketBuffer` - per-connection packet queues
//! - Connection and protocol state types
//!
//! NO SYSTEMS - just component definitions

use std::collections::{HashMap, VecDeque};

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use flecs_ecs::prelude::*;
use module_loader::register_module;

// ============================================================================
// Components
// ============================================================================

/// Packet received from async network layer
#[derive(Debug)]
pub struct IncomingPacket {
    pub connection_id: u64,
    pub packet_id: i32,
    pub data: Bytes,
}

/// Event signaling a connection has been closed
#[derive(Debug)]
pub struct DisconnectEvent {
    pub connection_id: u64,
}

/// Singleton: Receiver for disconnect events from async layer
#[derive(Component)]
pub struct DisconnectIngress {
    pub rx: Receiver<DisconnectEvent>,
}

/// Packet to send via async network layer
#[derive(Debug)]
pub struct OutgoingPacket {
    pub connection_id: u64,
    pub data: Bytes,
}

/// Singleton: Receiver for incoming packets from async layer
#[derive(Component)]
pub struct NetworkIngress {
    pub rx: Receiver<IncomingPacket>,
}

/// Singleton: Sender for outgoing packets to async layer
#[derive(Component)]
pub struct NetworkEgress {
    pub tx: Sender<OutgoingPacket>,
}

/// Tag: Entity is a network connection
#[derive(Component, Default)]
pub struct Connection;

/// Unique ID for routing packets to correct connection
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub u64);

/// Current protocol state of the connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Component)]
#[repr(C)]
pub enum ConnectionState {
    #[default]
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ProtocolState(pub ConnectionState);

/// Buffer for incoming/outgoing packets per connection
#[derive(Component, Default)]
pub struct PacketBuffer {
    pub incoming: VecDeque<(i32, Bytes)>,
    pub outgoing: VecDeque<Bytes>,
}

impl PacketBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_incoming(&mut self, packet_id: i32, data: Bytes) {
        self.incoming.push_back((packet_id, data));
    }

    pub fn pop_incoming(&mut self) -> Option<(i32, Bytes)> {
        self.incoming.pop_front()
    }

    pub fn push_outgoing(&mut self, data: Bytes) {
        self.outgoing.push_back(data);
    }

    pub fn pop_outgoing(&mut self) -> Option<Bytes> {
        self.outgoing.pop_front()
    }
}

/// Singleton: Maps connection IDs to their ECS entities
#[derive(Component, Default)]
pub struct ConnectionIndex {
    pub map: HashMap<u64, Entity>,
    /// Packets for newly created connections (deferred until next tick)
    pub pending_packets: Vec<(u64, i32, Bytes)>,
}

// ============================================================================
// Channel helpers
// ============================================================================

/// Channels for network I/O between async Tokio runtime and sync Flecs world
pub struct NetworkChannels {
    /// Sender for incoming packets (async -> ECS)
    pub ingress_tx: Sender<IncomingPacket>,
    /// Receiver for incoming packets (async -> ECS)
    pub ingress_rx: Receiver<IncomingPacket>,
    /// Sender for outgoing packets (ECS -> async)
    pub egress_tx: Sender<OutgoingPacket>,
    /// Receiver for outgoing packets (ECS -> async)
    pub egress_rx: Receiver<OutgoingPacket>,
    /// Sender for disconnect events (async -> ECS)
    pub disconnect_tx: Sender<DisconnectEvent>,
    /// Receiver for disconnect events (async -> ECS)
    pub disconnect_rx: Receiver<DisconnectEvent>,
}

impl NetworkChannels {
    /// Create a new set of network channels
    #[must_use]
    pub fn new() -> Self {
        let (ingress_tx, ingress_rx) = crossbeam_channel::unbounded();
        let (egress_tx, egress_rx) = crossbeam_channel::unbounded();
        let (disconnect_tx, disconnect_rx) = crossbeam_channel::unbounded();
        Self {
            ingress_tx,
            ingress_rx,
            egress_tx,
            egress_rx,
            disconnect_tx,
            disconnect_rx,
        }
    }
}

impl Default for NetworkChannels {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Module
// ============================================================================

/// Network components module - registers connection-related components only
#[derive(Component)]
pub struct NetworkComponentsModule;

impl Module for NetworkComponentsModule {
    fn module(world: &World) {
        world.module::<NetworkComponentsModule>("network::components");

        // Register components
        world.component::<Connection>();
        world.component::<ConnectionId>();
        world.component::<PacketBuffer>();
        world.component::<ProtocolState>();

        // Set up ConnectionIndex singleton
        world
            .component::<ConnectionIndex>()
            .add_trait::<flecs::Singleton>();
        world.set(ConnectionIndex::default());

        // NO SYSTEMS HERE - just components
    }
}

// ============================================================================
// Plugin exports
// ============================================================================

register_module! {
    name: "network-components",
    version: 1,
    module: NetworkComponentsModule,
    path: "::network::components",
}
