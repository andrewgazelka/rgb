use std::collections::VecDeque;

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use flecs_ecs::prelude::*;

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

/// Register ConnectionId with opaque serialization for flecs explorer
pub fn register_connection_id_meta(world: &World) {
    world
        .component::<ConnectionId>()
        .opaque_id(world.component_untyped().member(u64::id(), "id"))
        .serialize(|s: &Serializer, data: &ConnectionId| {
            s.member("id");
            s.value(&data.0);
            0
        });
}

/// Current protocol state of the connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Component)]
#[repr(C)]
#[flecs(meta)]
pub enum ConnectionState {
    #[default]
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Component, Debug, Clone, Copy, Default)]
#[flecs(meta)]
pub struct ProtocolState(pub ConnectionState);

/// Buffer for incoming/outgoing packets per connection
#[derive(Component, Default)]
pub struct PacketBuffer {
    pub incoming: VecDeque<(i32, Bytes)>,
    pub outgoing: VecDeque<Bytes>,
}

/// Register PacketBuffer with opaque serialization showing queue lengths
pub fn register_packet_buffer_meta(world: &World) {
    world
        .component::<PacketBuffer>()
        .opaque_id(
            world
                .component_untyped()
                .member(usize::id(), "incoming_count")
                .member(usize::id(), "outgoing_count"),
        )
        .serialize(|s: &Serializer, data: &PacketBuffer| {
            s.member("incoming_count");
            s.value(&data.incoming.len());
            s.member("outgoing_count");
            s.value(&data.outgoing.len());
            0
        });
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
