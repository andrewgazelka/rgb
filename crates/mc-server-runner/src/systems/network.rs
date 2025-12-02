//! Network systems - packet ingress/egress

use flecs_ecs::prelude::*;

use crate::components::{
    Connection, ConnectionId, ConnectionIndex, DisconnectIngress, NetworkEgress, NetworkIngress,
    OutgoingPacket, PacketBuffer, PendingPackets, ProtocolState,
};

/// System: Receive packets from network thread and route to connection entities
pub fn system_network_ingress(
    it: &TableIter<false, (&NetworkIngress, &mut PendingPackets, &mut ConnectionIndex)>,
) {
    let world = it.world();
    let ingress = &it.field::<NetworkIngress>(0).unwrap()[0];
    let pending = &mut it.field::<PendingPackets>(1).unwrap()[0];
    let conn_index = &mut it.field::<ConnectionIndex>(2).unwrap()[0];

    // Process pending packets from last tick
    let old_pending = core::mem::take(&mut pending.packets);
    for (conn_id, packet_id, data) in old_pending {
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

        if !conn_index.map.contains_key(&conn_id) {
            // New connection - create entity
            let name = format!("connection:{}", conn_id);
            let entity = world
                .entity_named(&name)
                .add::<Connection>()
                .set(ConnectionId(conn_id))
                .set(PacketBuffer::new())
                .set(ProtocolState::default())
                .id();
            conn_index.map.insert(conn_id, entity);

            // Queue packet for next tick
            pending
                .packets
                .push((conn_id, packet.packet_id, packet.data));
        } else {
            // Existing connection - route packet
            let entity = conn_index.map[&conn_id];
            let entity_view = world.entity_from_id(entity);
            let packet_id = packet.packet_id;
            let data = packet.data;
            let data_clone = data.clone();
            let routed = entity_view.try_get::<&mut PacketBuffer>(|buffer| {
                buffer.push_incoming(packet_id, data);
            });
            if routed.is_none() {
                pending.packets.push((conn_id, packet_id, data_clone));
            }
        }
    }
}

/// System: Handle disconnect events
pub fn system_handle_disconnects(
    it: &TableIter<false, (&DisconnectIngress, &mut ConnectionIndex)>,
) {
    let world = it.world();
    let disconnect = &it.field::<DisconnectIngress>(0).unwrap()[0];
    let conn_index = &mut it.field::<ConnectionIndex>(1).unwrap()[0];

    while let Ok(event) = disconnect.rx.try_recv() {
        let conn_id = event.connection_id;
        if let Some(entity) = conn_index.map.remove(&conn_id) {
            world.entity_from_id(entity).destruct();
        }
    }
}

/// Handle egress for a single connection
pub fn handle_egress(buffer: &mut PacketBuffer, conn_id: &ConnectionId, egress: &NetworkEgress) {
    while let Some(data) = buffer.pop_outgoing() {
        let _ = egress.tx.send(OutgoingPacket {
            connection_id: conn_id.0,
            data,
        });
    }
}
