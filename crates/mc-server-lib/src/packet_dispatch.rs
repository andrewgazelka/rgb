//! Packet dispatch system using Flecs components
//!
//! Handlers are entities with:
//! - `PacketHandler { handler: fn }` - the handler function
//! - `Priority(i32)` - execution order (lower = first)
//! - `HandlerFor { state, packet_id }` - which packet to handle
//!
//! Multiple handlers can handle the same packet (composable).
//! Handlers run in priority order.

use flecs_ecs::prelude::*;

use crate::components::{
    Connection, ConnectionState, HandlerFn, HandlerFor, PacketBuffer, PacketHandler, Priority,
    ProtocolState,
};

/// Extension trait for World to register packet handlers
pub trait PacketHandlerRegistration {
    /// Register a handler for a specific packet type
    ///
    /// # Arguments
    /// * `name` - Handler name (for debugging)
    /// * `state` - Protocol state this handler applies to
    /// * `packet_id` - Packet ID to handle
    /// * `priority` - Execution priority (lower = runs first)
    /// * `handler` - Handler function
    fn register_handler(
        &self,
        name: &str,
        state: ConnectionState,
        packet_id: i32,
        priority: i32,
        handler: HandlerFn,
    ) -> EntityView<'_>;
}

impl PacketHandlerRegistration for World {
    fn register_handler(
        &self,
        name: &str,
        state: ConnectionState,
        packet_id: i32,
        priority: i32,
        handler: HandlerFn,
    ) -> EntityView<'_> {
        let handler_name = format!("handler::{name}");
        self.entity_named(&handler_name)
            .set(PacketHandler { handler })
            .set(Priority(priority))
            .set(HandlerFor { state, packet_id })
    }
}

/// Packet dispatch module
#[derive(Component)]
pub struct PacketDispatchModule;

impl Module for PacketDispatchModule {
    fn module(world: &World) {
        world.module::<PacketDispatchModule>("packet_dispatch");

        // Dispatch system - processes packets for all connections
        // For each packet, queries all handlers that match (state, packet_id)
        world
            .system_named::<(&mut PacketBuffer, &ProtocolState)>("DispatchPackets")
            .with(Connection)
            .run(|mut it| {
                while it.next() {
                    let world_ref = it.world();

                    for i in it.iter() {
                        let buffer = &mut it.field_mut::<PacketBuffer>(0)[i];
                        let state = it.field::<ProtocolState>(1)[i].0;
                        let conn_entity = it.entity(i);

                        // Process incoming packets that have registered handlers
                        // Peek at packets without consuming, only pop if we have handlers
                        loop {
                            // Peek at the front packet
                            let Some((packet_id, _)) = buffer.incoming.front() else {
                                break;
                            };
                            let packet_id = *packet_id;

                            // Query all handlers for this (state, packet_id)
                            let mut handlers: Vec<(i32, HandlerFn)> = Vec::new();

                            world_ref.each_entity::<(&PacketHandler, &Priority, &HandlerFor)>(
                                |_handler_entity, (handler, priority, handler_for)| {
                                    if handler_for.state == state
                                        && handler_for.packet_id == packet_id
                                    {
                                        handlers.push((priority.0, handler.handler));
                                    }
                                },
                            );

                            // If no handlers registered, leave packet for other systems
                            if handlers.is_empty() {
                                break;
                            }

                            // Now actually consume the packet
                            let (_, data) = buffer.pop_incoming().unwrap();

                            // Sort by priority (lower first)
                            handlers.sort_by_key(|(priority, _)| *priority);

                            // Execute handlers in order
                            for (_priority, handler_fn) in handlers {
                                handler_fn(conn_entity, &data);
                            }
                        }
                    }
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::sync::atomic::{AtomicU32, Ordering};

    static HANDLER_CALL_COUNT: AtomicU32 = AtomicU32::new(0);
    static HANDLER_ORDER: AtomicU32 = AtomicU32::new(0);

    fn test_handler_a(_entity: EntityView<'_>, _data: &[u8]) {
        HANDLER_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        // Record order - this should be called first (priority 0)
        assert_eq!(HANDLER_ORDER.fetch_add(1, Ordering::SeqCst), 0);
    }

    fn test_handler_b(_entity: EntityView<'_>, _data: &[u8]) {
        HANDLER_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        // Record order - this should be called second (priority 10)
        assert_eq!(HANDLER_ORDER.fetch_add(1, Ordering::SeqCst), 1);
    }

    #[test]
    fn test_handler_registration_and_dispatch() {
        // Reset counters
        HANDLER_CALL_COUNT.store(0, Ordering::SeqCst);
        HANDLER_ORDER.store(0, Ordering::SeqCst);

        let world = World::new();

        // Import the dispatch module
        world.import::<PacketDispatchModule>();

        // Register two handlers for the same packet with different priorities
        world.register_handler(
            "TestHandlerA",
            ConnectionState::Play,
            0x42, // arbitrary packet ID
            0,    // priority 0 (runs first)
            test_handler_a,
        );
        world.register_handler(
            "TestHandlerB",
            ConnectionState::Play,
            0x42,
            10, // priority 10 (runs second)
            test_handler_b,
        );

        // Create a test connection entity
        let conn = world
            .entity_named("test_connection")
            .add(Connection)
            .set(ProtocolState(ConnectionState::Play))
            .set(PacketBuffer::default());

        // Add a packet to the buffer
        conn.get::<&mut PacketBuffer>(|buffer| {
            buffer.push_incoming(0x42, Bytes::from_static(b"test data"));
        });

        // Progress the world to run systems
        world.progress();

        // Verify both handlers were called
        assert_eq!(HANDLER_CALL_COUNT.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_handler_state_filtering() {
        fn counting_handler(_entity: EntityView<'_>, _data: &[u8]) {
            // Handler shouldn't be called for wrong state
            panic!("Handler should not be called for wrong state!");
        }

        let world = World::new();
        world.import::<PacketDispatchModule>();

        // Register handler only for Play state
        world.register_handler(
            "PlayOnlyHandler",
            ConnectionState::Play,
            0x99,
            0,
            counting_handler,
        );

        // Create connection in Login state (should NOT trigger handler)
        let conn = world
            .entity_named("login_connection")
            .add(Connection)
            .set(ProtocolState(ConnectionState::Login))
            .set(PacketBuffer::default());

        conn.get::<&mut PacketBuffer>(|buffer| {
            buffer.push_incoming(0x99, Bytes::from_static(b"data"));
        });

        world.progress();

        // Packet should NOT be consumed since no handler is registered for Login state
        // This allows other systems to process it
        conn.get::<&PacketBuffer>(|buffer| {
            assert_eq!(buffer.incoming.len(), 1, "Packet should NOT be consumed");
        });
    }
}
