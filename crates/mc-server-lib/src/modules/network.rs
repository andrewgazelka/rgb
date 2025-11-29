use flecs_ecs::prelude::*;

use crate::components::{
    Connection, ConnectionId, NetworkEgress, NetworkIngress, OutgoingPacket, PacketBuffer,
};

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

        // Register singletons
        world
            .component::<NetworkIngress>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<NetworkEgress>()
            .add_trait::<flecs::Singleton>();

        // INGRESS: First system in tick (OnLoad phase)
        // Reads packets from async channel and distributes to connection entities
        world
            .system_named::<(&mut PacketBuffer, &ConnectionId, &NetworkIngress)>("NetworkIngress")
            .kind(id::<flecs::pipeline::OnLoad>())
            .with(Connection)
            .each(|(buffer, conn_id, ingress)| {
                // Drain all packets for this connection from the channel
                while let Ok(packet) = ingress.rx.try_recv() {
                    if packet.connection_id == conn_id.0 {
                        buffer.push_incoming(packet.packet_id, packet.data);
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
                    let _ = egress.tx.send(OutgoingPacket {
                        connection_id: conn_id.0,
                        data,
                    });
                }
            });
    }
}
