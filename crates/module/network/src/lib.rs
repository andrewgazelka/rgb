//! Network module - handles packet ingress/egress and connection management
//!
//! This module provides:
//! - `NetworkIngress` / `NetworkEgress` - channel endpoints for async I/O
//! - `ConnectionIndex` - maps connection IDs to ECS entities
//! - `PacketBuffer` - per-connection packet queues
//! - Systems for routing packets to/from entities

use std::collections::{HashMap, VecDeque};

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use flecs_ecs::prelude::*;

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
// Module
// ============================================================================

/// Network module - handles ingress/egress at start/end of tick
#[derive(Component)]
pub struct NetworkModule;

impl Module for NetworkModule {
    fn module(world: &World) {
        world.module::<NetworkModule>("network");

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

        // INGRESS: First system in tick (OnLoad phase)
        world
            .system_named::<(&NetworkIngress, &mut ConnectionIndex)>("NetworkIngress")
            .kind(id::<flecs::pipeline::OnLoad>())
            .run(|mut it| {
                while it.next() {
                    let ingress = &it.field::<NetworkIngress>(0)[0];
                    let conn_index = &mut it.field_mut::<ConnectionIndex>(1)[0];
                    let world = it.world();

                    // Process pending packets from last tick
                    let pending = std::mem::take(&mut conn_index.pending_packets);
                    for (conn_id, packet_id, data) in pending {
                        if let Some(&entity) = conn_index.map.get(&conn_id) {
                            let entity_view = world.entity_from_id(entity);
                            entity_view.try_get::<&mut PacketBuffer>(|buffer| {
                                buffer.push_incoming(packet_id, data);
                            });
                        }
                    }

                    // Drain all packets from the channel
                    while let Ok(packet) = ingress.rx.try_recv() {
                        let conn_id = packet.connection_id;

                        let is_new = !conn_index.map.contains_key(&conn_id);
                        if is_new {
                            let name = format!("connection:{}", conn_id);
                            let entity = world
                                .entity_named(&name)
                                .add(Connection)
                                .set(ConnectionId(conn_id))
                                .set(PacketBuffer::new())
                                .set(ProtocolState::default())
                                .id();
                            conn_index.map.insert(conn_id, entity);

                            // Queue packet for next tick
                            conn_index.pending_packets.push((
                                conn_id,
                                packet.packet_id,
                                packet.data,
                            ));
                        } else {
                            let entity = conn_index.map[&conn_id];
                            let entity_view = world.entity_from_id(entity);
                            let packet_id = packet.packet_id;
                            let data = packet.data;
                            let data_clone = data.clone();
                            let routed = entity_view.try_get::<&mut PacketBuffer>(|buffer| {
                                buffer.push_incoming(packet_id, data);
                            });
                            if routed.is_none() {
                                conn_index
                                    .pending_packets
                                    .push((conn_id, packet_id, data_clone));
                            }
                        }
                    }
                }
            });

        // DISCONNECT: Handle disconnect events
        world
            .system_named::<(&DisconnectIngress, &mut ConnectionIndex)>("HandleDisconnects")
            .kind(id::<flecs::pipeline::OnLoad>())
            .run(|mut it| {
                while it.next() {
                    let disconnect = &it.field::<DisconnectIngress>(0)[0];
                    let conn_index = &mut it.field_mut::<ConnectionIndex>(1)[0];
                    let world = it.world();

                    while let Ok(event) = disconnect.rx.try_recv() {
                        let conn_id = event.connection_id;
                        if let Some(entity) = conn_index.map.remove(&conn_id) {
                            world.entity_from_id(entity).destruct();
                        }
                        conn_index
                            .pending_packets
                            .retain(|(id, _, _)| *id != conn_id);
                    }
                }
            });

        // EGRESS: Last system in tick (OnStore phase)
        world
            .system_named::<(&mut PacketBuffer, &ConnectionId, &NetworkEgress)>("NetworkEgress")
            .kind(id::<flecs::pipeline::OnStore>())
            .with(Connection)
            .each(|(buffer, conn_id, egress)| {
                while let Some(data) = buffer.pop_outgoing() {
                    let _ = egress.tx.send(OutgoingPacket {
                        connection_id: conn_id.0,
                        data,
                    });
                }
            });
    }
}

// ============================================================================
// Plugin exports
// ============================================================================

module_loader::register_module! {
    name: "network",
    version: 1,
    module: NetworkModule,
    path: "::network",
}
