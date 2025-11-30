//! Network systems module - packet ingress/egress and connection management
//!
//! This module provides systems that operate on network components.
//! Depends on `module-network-components` for component definitions.

use flecs_ecs::prelude::*;
use module_loader::register_plugin;
use module_network_components::{
    Connection, ConnectionId, ConnectionIndex, DisconnectIngress, NetworkComponentsModule,
    NetworkEgress, NetworkIngress, OutgoingPacket, PacketBuffer, ProtocolState,
};

// ============================================================================
// Module
// ============================================================================

/// Network systems module - handles ingress/egress at start/end of tick
#[derive(Component)]
pub struct NetworkSystemsModule;

impl Module for NetworkSystemsModule {
    fn module(world: &World) {
        world.module::<NetworkSystemsModule>("network::systems");

        // Import component module (ensures components exist)
        world.import::<NetworkComponentsModule>();

        // INGRESS: First system in tick (OnLoad phase)
        world
            .system_named::<()>("NetworkIngress")
            .kind(id::<flecs::pipeline::OnLoad>())
            .run(|mut it| {
                while it.next() {
                    let world = it.world();

                    // Get singletons directly from world
                    world.get::<(&NetworkIngress, &mut ConnectionIndex)>(
                        |(ingress, conn_index)| {
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
                                    let routed =
                                        entity_view.try_get::<&mut PacketBuffer>(|buffer| {
                                            buffer.push_incoming(packet_id, data);
                                        });
                                    if routed.is_none() {
                                        conn_index
                                            .pending_packets
                                            .push((conn_id, packet_id, data_clone));
                                    }
                                }
                            }
                        },
                    );
                }
            });

        // DISCONNECT: Handle disconnect events
        world
            .system_named::<()>("HandleDisconnects")
            .kind(id::<flecs::pipeline::OnLoad>())
            .run(|mut it| {
                while it.next() {
                    let world = it.world();

                    world.get::<(&DisconnectIngress, &mut ConnectionIndex)>(
                        |(disconnect, conn_index)| {
                            while let Ok(event) = disconnect.rx.try_recv() {
                                let conn_id = event.connection_id;
                                if let Some(entity) = conn_index.map.remove(&conn_id) {
                                    world.entity_from_id(entity).destruct();
                                }
                                conn_index
                                    .pending_packets
                                    .retain(|(id, _, _)| *id != conn_id);
                            }
                        },
                    );
                }
            });

        // EGRESS: Last system in tick (OnStore phase)
        world
            .system_named::<(&mut PacketBuffer, &ConnectionId)>("NetworkEgress")
            .kind(id::<flecs::pipeline::OnStore>())
            .with(Connection)
            .each_entity(|e, (buffer, conn_id)| {
                let world = e.world();
                world.get::<&NetworkEgress>(|egress| {
                    while let Some(data) = buffer.pop_outgoing() {
                        let _ = egress.tx.send(OutgoingPacket {
                            connection_id: conn_id.0,
                            data,
                        });
                    }
                });
            });
    }
}

// ============================================================================
// Plugin exports
// ============================================================================

register_module! {
    name: "network-systems",
    version: 1,
    module: NetworkSystemsModule,
    path: "::network::systems",
}
