//! Event marker trait.
//!
//! Events are just components that can be sent to entities.
//! The Event trait marks a type as being usable as an event.

use core::any::TypeId;

/// Marker trait for event components.
///
/// Any type implementing this can be sent as an event via `world.send()`.
/// Events are just components attached to event entities.
///
/// # Safety
///
/// This trait is safe to implement. The type must be `Send + Sync + 'static`.
pub trait Event: Send + Sync + 'static {
    /// Get the TypeId of this event type.
    fn type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

// Blanket implementation: any Send + Sync + 'static type can be an event
impl<T: Send + Sync + 'static> Event for T {}

#[cfg(test)]
mod tests {
    use super::*;

    struct Damage {
        amount: f32,
    }

    struct Explosion {
        radius: f32,
    }

    #[test]
    fn test_event_type_id() {
        assert_eq!(Damage::type_id(), TypeId::of::<Damage>());
        assert_ne!(Damage::type_id(), Explosion::type_id());
    }

    #[test]
    fn test_any_type_is_event() {
        fn assert_event<T: Event>() {}

        assert_event::<Damage>();
        assert_event::<Explosion>();
        assert_event::<i32>();
        assert_event::<String>();
    }
}
