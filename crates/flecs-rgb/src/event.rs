//! Custom event handler system using Flecs relations
//!
//! Events are dispatched to handlers registered via relations.
//! This is a custom system, NOT using Flecs observers.

use core::any::TypeId;
use core::ffi::c_void;

use flecs_ecs::prelude::*;

use crate::ScopedWorld;

/// Marker relation for event handlers.
/// `handler_entity.add((EventHandler, target_entity))` means handler handles events for target.
#[derive(Component)]
pub struct EventHandler;

/// Type-erased handler information stored on handler entities
#[derive(Component, Clone)]
pub struct HandlerInfo {
    /// TypeId of the event this handler processes
    pub event_type_id: TypeId,
    /// Type-erased handler function pointer
    /// Signature: fn(event_ptr: *const c_void, scoped: &ScopedWorld, target: EntityView)
    pub handler_fn: fn(*const c_void, &ScopedWorld<'_>, EntityView<'_>),
    /// Size of the event type (for safety checks)
    pub event_size: usize,
    /// Name of the event type (for debugging)
    pub event_name: &'static str,
}

/// Trait for event types
pub trait Event: 'static + Sized {
    /// Get the event's type name for debugging
    fn event_name() -> &'static str {
        core::any::type_name::<Self>()
    }
}

/// Extension trait for World to work with events
pub trait EventWorldExt {
    /// Register an event handler for a target entity
    ///
    /// The handler function must be a bare function pointer (not a closure).
    /// It receives:
    /// - `event_ptr`: Pointer to the event data (cast to concrete type inside handler)
    /// - `scoped`: ScopedWorld for accessing/modifying components
    /// - `target`: The entity that received the event
    ///
    /// # Example
    /// ```ignore
    /// fn on_attack(event_ptr: *const c_void, scoped: &ScopedWorld, target: EntityView) {
    ///     let attack = unsafe { &*(event_ptr as *const Attack) };
    ///     // Handle attack
    /// }
    ///
    /// world.register_handler::<Attack>(player, on_attack);
    /// ```
    fn register_handler<E: Event>(
        &self,
        target: EntityView<'_>,
        handler: fn(*const c_void, &ScopedWorld<'_>, EntityView<'_>),
    ) -> EntityView<'_>;

    /// Dispatch an event to all handlers registered for the target
    fn dispatch<E: Event>(&self, target: EntityView<'_>, event: &E, scoped: &ScopedWorld<'_>);
}

impl EventWorldExt for World {
    fn register_handler<E: Event>(
        &self,
        target: EntityView<'_>,
        handler: fn(*const c_void, &ScopedWorld<'_>, EntityView<'_>),
    ) -> EntityView<'_> {
        self.entity()
            .set(HandlerInfo {
                event_type_id: TypeId::of::<E>(),
                handler_fn: handler,
                event_size: core::mem::size_of::<E>(),
                event_name: E::event_name(),
            })
            .add((EventHandler, target))
    }

    fn dispatch<E: Event>(&self, target: EntityView<'_>, event: &E, scoped: &ScopedWorld<'_>) {
        let event_type_id = TypeId::of::<E>();
        let event_ptr = core::ptr::from_ref(event).cast::<c_void>();

        // Query all handlers for this target
        self.query::<&HandlerInfo>()
            .with((EventHandler, target))
            .build()
            .each(|info| {
                if info.event_type_id == event_type_id {
                    (info.handler_fn)(event_ptr, scoped, target);
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;
    use std::sync::atomic::{AtomicU32, Ordering};

    // Test event
    struct Attack {
        damage: u32,
    }

    impl Event for Attack {}

    // Global counter for test
    static DAMAGE_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn on_attack(event_ptr: *const c_void, _scoped: &ScopedWorld<'_>, _target: EntityView<'_>) {
        let event = unsafe { &*event_ptr.cast::<Attack>() };
        DAMAGE_COUNTER.fetch_add(event.damage, Ordering::Relaxed);
    }

    #[test]
    fn test_event_dispatch() {
        // Reset counter
        DAMAGE_COUNTER.store(0, Ordering::Relaxed);

        let world = World::new();

        // Create a target entity with position
        let player = world.entity().set(Position::new(0.0, 64.0, 0.0));

        // Register handler
        let _handler = world.register_handler::<Attack>(player, on_attack);

        // Create a scoped world for the handler
        let scoped = ScopedWorld::new((&world).world(), (0, 0));

        // Dispatch events
        world.dispatch(player, &Attack { damage: 10 }, &scoped);
        world.dispatch(player, &Attack { damage: 25 }, &scoped);

        assert_eq!(DAMAGE_COUNTER.load(Ordering::Relaxed), 35);
    }

    // Another event type
    struct Heal {
        amount: u32,
    }

    impl Event for Heal {}

    static HEAL_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn on_heal(event_ptr: *const c_void, _scoped: &ScopedWorld<'_>, _target: EntityView<'_>) {
        let event = unsafe { &*event_ptr.cast::<Heal>() };
        HEAL_COUNTER.fetch_add(event.amount, Ordering::Relaxed);
    }

    #[test]
    fn test_multiple_event_types() {
        // Reset counters
        DAMAGE_COUNTER.store(0, Ordering::Relaxed);
        HEAL_COUNTER.store(0, Ordering::Relaxed);

        let world = World::new();

        let player = world.entity().set(Position::new(0.0, 64.0, 0.0));

        // Register handlers for different event types
        let _h1 = world.register_handler::<Attack>(player, on_attack);
        let _h2 = world.register_handler::<Heal>(player, on_heal);

        let scoped = ScopedWorld::new((&world).world(), (0, 0));

        // Dispatch different events
        world.dispatch(player, &Attack { damage: 50 }, &scoped);
        world.dispatch(player, &Heal { amount: 30 }, &scoped);

        assert_eq!(DAMAGE_COUNTER.load(Ordering::Relaxed), 50);
        assert_eq!(HEAL_COUNTER.load(Ordering::Relaxed), 30);
    }
}
