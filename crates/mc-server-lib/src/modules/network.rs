use std::collections::HashMap;

use bytes::Bytes;
use flecs_ecs::prelude::*;

use crate::components::{
    Connection, ConnectionId, NetworkEgress, NetworkIngress, OutgoingPacket, PacketBuffer,
    ProtocolState,
};

/// Singleton: Maps connection IDs to their ECS entities
#[derive(Component, Default)]
pub struct ConnectionIndex {
    pub map: HashMap<u64, Entity>,
    /// Packets for newly created connections (deferred until next tick when components are ready)
    pub pending_packets: Vec<(u64, i32, Bytes)>,
}

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

        // NetworkIngress/NetworkEgress/ConnectionIndex singleton traits are registered in create_world()

        // INGRESS: First system in tick (OnLoad phase)
        // Drains all packets from channel, creates connection entities as needed, routes to buffers
        world
            .system_named::<(&NetworkIngress, &mut ConnectionIndex)>("NetworkIngress")
            .kind(id::<flecs::pipeline::OnLoad>())
            .run(|mut it| {
                while it.next() {
                    let ingress = &it.field::<NetworkIngress>(0)[0];
                    let conn_index = &mut it.field_mut::<ConnectionIndex>(1)[0];
                    let world = it.world();

                    // First, process any pending packets from last tick (for newly created entities)
                    let pending = std::mem::take(&mut conn_index.pending_packets);
                    for (conn_id, packet_id, data) in &pending {
                        tracing::debug!(
                            "Processing pending packet: conn_id={}, packet_id={}",
                            conn_id,
                            packet_id
                        );
                    }
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
                        tracing::debug!(
                            "Ingress received packet: conn_id={}, packet_id={}",
                            conn_id,
                            packet.packet_id
                        );

                        // Get or create connection entity
                        let is_new = !conn_index.map.contains_key(&conn_id);
                        if is_new {
                            // Create new connection entity
                            let entity = world
                                .entity()
                                .add(Connection)
                                .set(ConnectionId(conn_id))
                                .set(PacketBuffer::new())
                                .set(ProtocolState::default())
                                .id();
                            conn_index.map.insert(conn_id, entity);
                            tracing::debug!("Created connection entity for conn_id={}", conn_id);

                            // Queue packet for next tick (components are deferred)
                            conn_index.pending_packets.push((
                                conn_id,
                                packet.packet_id,
                                packet.data,
                            ));
                        } else {
                            // Route packet to existing entity's buffer
                            let entity = conn_index.map[&conn_id];
                            let entity_view = world.entity_from_id(entity);
                            let packet_id = packet.packet_id;
                            let data = packet.data;
                            let data_clone = data.clone();
                            let routed = entity_view.try_get::<&mut PacketBuffer>(|buffer| {
                                buffer.push_incoming(packet_id, data);
                                tracing::debug!(
                                    "Routed packet to buffer: conn_id={}, packet_id={}",
                                    conn_id,
                                    packet_id
                                );
                            });
                            if routed.is_none() {
                                tracing::warn!(
                                    "Failed to route packet (no buffer): conn_id={}, packet_id={}",
                                    conn_id,
                                    packet_id
                                );
                                // Buffer not ready yet, queue for next tick
                                conn_index
                                    .pending_packets
                                    .push((conn_id, packet_id, data_clone));
                            }
                        }
                    }
                }
            });

        // EGRESS: Last system in tick (OnStore phase)
        // Sends buffered packets to async channel
        world
            .system_named::<(&mut PacketBuffer, &ConnectionId, &NetworkEgress)>("NetworkEgress")
            .kind(id::<flecs::pipeline::OnStore>())
            .with(Connection)
            .each(|(buffer, conn_id, egress)| {
                while let Some(data) = buffer.pop_outgoing() {
                    tracing::debug!(
                        "Egress sending packet: conn_id={}, len={}",
                        conn_id.0,
                        data.len()
                    );
                    let _ = egress.tx.send(OutgoingPacket {
                        connection_id: conn_id.0,
                        data,
                    });
                }
            });
    }
}
