//! Network systems - packet ingress/egress

use rgb_ecs::{Entity, World};

use crate::components::{
    Connection, ConnectionId, ConnectionIndex, DisconnectIngress, NetworkEgress, NetworkIngress,
    OutgoingPacket, PacketBuffer, ProtocolState,
};

/// System: Receive packets from network thread and route to connection entities
pub fn system_network_ingress(world: &mut World) {
    // Get global state
    let Some(ingress) = world.get::<NetworkIngress>(Entity::WORLD) else {
        return;
    };
    let Some(mut conn_index) = world.get::<ConnectionIndex>(Entity::WORLD) else {
        return;
    };

    // Process pending packets from last tick
    let pending = core::mem::take(&mut conn_index.pending_packets);
    for (conn_id, packet_id, data) in pending {
        if let Some(&entity) = conn_index.map.get(&conn_id) {
            if let Some(mut buffer) = world.get::<PacketBuffer>(entity) {
                buffer.push_incoming(packet_id, data);
                world.update(entity, buffer);
            }
        }
    }

    // Drain all packets from the channel
    while let Ok(packet) = ingress.rx.try_recv() {
        let conn_id = packet.connection_id;

        if !conn_index.map.contains_key(&conn_id) {
            // New connection - create entity
            let mut buffer = PacketBuffer::new();
            buffer.push_incoming(packet.packet_id, packet.data);

            let entity = world.spawn_empty();
            world.insert(entity, Connection);
            world.insert(entity, ConnectionId(conn_id));
            world.insert(entity, buffer);
            world.insert(entity, ProtocolState::default());

            conn_index.map.insert(conn_id, entity);
        } else {
            // Existing connection - route packet
            let entity = conn_index.map[&conn_id];
            if let Some(mut buffer) = world.get::<PacketBuffer>(entity) {
                buffer.push_incoming(packet.packet_id, packet.data.clone());
                world.update(entity, buffer);
            } else {
                // Entity doesn't have buffer yet, defer
                conn_index
                    .pending_packets
                    .push((conn_id, packet.packet_id, packet.data));
            }
        }
    }

    // Write back connection index
    world.update(Entity::WORLD, conn_index);
}

/// System: Handle disconnect events
pub fn system_handle_disconnects(world: &mut World) {
    let Some(disconnect) = world.get::<DisconnectIngress>(Entity::WORLD) else {
        return;
    };
    let Some(mut conn_index) = world.get::<ConnectionIndex>(Entity::WORLD) else {
        return;
    };

    while let Ok(event) = disconnect.rx.try_recv() {
        let conn_id = event.connection_id;
        if let Some(entity) = conn_index.map.remove(&conn_id) {
            world.despawn(entity);
        }
        conn_index
            .pending_packets
            .retain(|(id, _, _)| *id != conn_id);
    }

    world.update(Entity::WORLD, conn_index);
}

/// System: Send outgoing packets to network thread
pub fn system_network_egress(world: &mut World) {
    let Some(egress) = world.get::<NetworkEgress>(Entity::WORLD) else {
        return;
    };
    let Some(conn_index) = world.get::<ConnectionIndex>(Entity::WORLD) else {
        return;
    };

    // Iterate over all connections
    for (&conn_id, &entity) in &conn_index.map {
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
