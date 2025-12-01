//! Observer system for handling events.

use core::any::TypeId;
use core::marker::PhantomData;

use rgb_ecs::{Entity, World};

/// Unique identifier for a registered observer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObserverId(pub(crate) u32);

impl ObserverId {
    /// Create a new observer ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Type-erased observer function.
pub(crate) type ObserverFn = Box<dyn Fn(&mut World, Entity, *const u8) + Send + Sync>;

/// Metadata for a registered observer.
pub struct ObserverInfo {
    /// Unique ID
    pub id: ObserverId,
    /// Event type this observer handles
    pub event_type_id: TypeId,
    /// Event type name for debugging
    pub event_name: &'static str,
    /// The observer function (type-erased)
    pub(crate) callback: ObserverFn,
}

impl core::fmt::Debug for ObserverInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ObserverInfo")
            .field("id", &self.id)
            .field("event_type_id", &self.event_type_id)
            .field("event_name", &self.event_name)
            .finish_non_exhaustive()
    }
}

/// Trait for observer functions.
///
/// Observers are callbacks that run when specific events occur.
pub trait Observer<E>: Send + Sync + 'static {
    /// Handle the event.
    fn observe(&self, world: &mut World, target: Entity, event: &E);
}

// Implement Observer for closures
impl<E, F> Observer<E> for F
where
    E: 'static,
    F: Fn(&mut World, Entity, &E) + Send + Sync + 'static,
{
    fn observe(&self, world: &mut World, target: Entity, event: &E) {
        self(world, target, event);
    }
}

/// Builder for creating observers with type safety.
pub struct ObserverBuilder<E> {
    _marker: PhantomData<E>,
}

impl<E: Send + Sync + 'static> ObserverBuilder<E> {
    /// Create a new observer builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Build an observer from a callback function.
    pub fn build<F>(self, callback: F) -> ObserverInfo
    where
        F: Fn(&mut World, Entity, &E) + Send + Sync + 'static,
    {
        ObserverInfo {
            id: ObserverId::new(0), // ID assigned during registration
            event_type_id: TypeId::of::<E>(),
            event_name: core::any::type_name::<E>(),
            callback: Box::new(move |world, target, event_ptr| {
                // SAFETY: event_ptr points to a valid E, guaranteed by caller
                let event = unsafe { &*event_ptr.cast::<E>() };
                callback(world, target, event);
            }),
        }
    }
}

impl<E: Send + Sync + 'static> Default for ObserverBuilder<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct TestEvent {
        value: i32,
    }

    #[test]
    fn test_observer_builder() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let info = ObserverBuilder::<TestEvent>::new().build(move |_world, _target, event| {
            counter_clone.fetch_add(event.value as u32, Ordering::SeqCst);
        });

        assert_eq!(info.event_type_id, TypeId::of::<TestEvent>());

        // Call the observer
        let mut world = World::new();
        let event = TestEvent { value: 42 };
        (info.callback)(
            &mut world,
            Entity::WORLD,
            core::ptr::from_ref(&event).cast(),
        );

        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }
}
