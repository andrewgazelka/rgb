//! Network systems - packet ingress/egress

use rgb_ecs::{Entity, World};

use crate::components::{
    Connection, ConnectionId, DisconnectIngress, NetworkEgress, NetworkIngress, OutgoingPacket,
    PacketBuffer, PendingPackets, ProtocolState,
};

/// System: Receive packets from network thread and route to connection entities
pub fn system_network_ingress(world: &mut World) {
    // Get global state
    let Some(ingress) = world.get::<NetworkIngress>(Entity::WORLD) else {
        return;
    };
    let mut pending = world
        .get::<PendingPackets>(Entity::WORLD)
        .unwrap_or_default();

    // Process pending packets from last tick
    let old_pending = core::mem::take(&mut pending.packets);
    for (conn_id, packet_id, data) in old_pending {
        let key = ConnectionId(conn_id).to_key();
        if let Some(entity) = world.lookup(&key) {
            if let Some(mut buffer) = world.get::<PacketBuffer>(entity) {
                buffer.push_incoming(packet_id, data);
                world.update(entity, buffer);
            }
        }
    }

    // Drain all packets from the channel
    while let Ok(packet) = ingress.rx.try_recv() {
        let conn_id = ConnectionId(packet.connection_id);
        let key = conn_id.to_key();

        if world.lookup(&key).is_none() {
            // New connection - create entity with named key
            let mut buffer = PacketBuffer::new();
            buffer.push_incoming(packet.packet_id, packet.data);

            let entity = world.entity_named(&key);
            world.insert(entity, Connection);
            world.insert(entity, conn_id);
            world.insert(entity, buffer);
            world.insert(entity, ProtocolState::default());
        } else {
            // Existing connection - route packet
            let entity = world.lookup(&key).unwrap();
            if let Some(mut buffer) = world.get::<PacketBuffer>(entity) {
                buffer.push_incoming(packet.packet_id, packet.data.clone());
                world.update(entity, buffer);
            } else {
                // Entity doesn't have buffer yet, defer
                pending
                    .packets
                    .push((packet.connection_id, packet.packet_id, packet.data));
            }
        }
    }

    // Write back pending packets
    world.update(Entity::WORLD, pending);
}

/// System: Handle disconnect events
pub fn system_handle_disconnects(world: &mut World) {
    let Some(disconnect) = world.get::<DisconnectIngress>(Entity::WORLD) else {
        return;
    };
    let mut pending = world
        .get::<PendingPackets>(Entity::WORLD)
        .unwrap_or_default();

    while let Ok(event) = disconnect.rx.try_recv() {
        let conn_id = ConnectionId(event.connection_id);
        let key = conn_id.to_key();

        // Despawn the connection entity (this also removes it from name index)
        if let Some(entity) = world.lookup(&key) {
            world.despawn(entity);
        }

        // Remove any pending packets for this connection
        pending.packets.retain(|(id, _, _)| *id != conn_id.0);
    }

    world.update(Entity::WORLD, pending);
}

/// System: Send outgoing packets to network thread
pub fn system_network_egress(world: &mut World) {
    let Some(egress) = world.get::<NetworkEgress>(Entity::WORLD) else {
        return;
    };

    // Query all connection entities
    let connections: Vec<_> = world
        .query::<ConnectionId>()
        .map(|(entity, conn_id)| (entity, conn_id.0))
        .collect();

    // Send outgoing packets for each connection
    for (entity, conn_id) in connections {
        if let Some(mut buffer) = world.get::<PacketBuffer>(entity) {
            while let Some(data) = buffer.pop_outgoing() {
                let _ = egress.tx.send(OutgoingPacket {
                    connection_id: conn_id,
                    data,
                });
            }
            world.update(entity, buffer);
        }
    }
}
