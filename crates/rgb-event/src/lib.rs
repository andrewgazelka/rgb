#![allow(unsafe_code)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::redundant_pub_crate)]
#![allow(clippy::unused_self)]
#![allow(clippy::needless_collect)]

//! RGB Event System
//!
//! Event-driven architecture with RGB spatial parallelism.
//!
//! # Core Concept: Events as Entities
//!
//! Events are just entities with special components:
//! - Event data component (e.g., `Damage`, `Explosion`)
//! - `Target` component pointing to the target entity
//! - Or `Position` component for positional events
//!
//! # Scheduling
//!
//! - Events targeting `Entity::WORLD` → Global queue (sequential)
//! - Events with `Position` → RGB queue by `position.cell_color()` (parallel)
//! - Events with `Target(entity)` → RGB queue by target's position
//!
//! # Example
//!
//! ```ignore
//! // Send damage event to a player (uses player's position for RGB scheduling)
//! world.send(player, Damage { amount: 10.0 });
//!
//! // Send explosion at position (uses position for RGB scheduling)
//! world.send_at(Position::new(100.0, 64.0, 200.0), Explosion { radius: 5.0 });
//!
//! // Send global tick event (sequential, no position)
//! world.send(Entity::WORLD, Tick { delta: 0.05 });
//!
//! // Register observer for Damage events
//! world.observe::<Damage, _>(|event, target, world| {
//!     let mut health = world.get::<Health>(target).unwrap();
//!     health.current -= event.amount;
//!     world.update(target, health);
//! });
//! ```

mod color;
mod event;
mod observer;
mod queue;
mod world_ext;

use rgb_ecs::{Plugin, World};

pub use color::cell_color;
pub use event::Event;
pub use observer::{Observer, ObserverId};
pub use queue::EventQueue;
pub use world_ext::{EventSystem, EventWorldExt, Position, Target};

/// Plugin to add the event system to a World.
///
/// # Example
///
/// ```ignore
/// use rgb_ecs::World;
/// use rgb_event::EventPlugin;
///
/// let mut world = World::new();
/// world.add_plugin(EventPlugin);
///
/// // Now you can use events
/// world.send(Entity::WORLD, MyEvent { value: 42 });
/// world.flush_events();
/// ```
pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, world: &mut World) {
        world_ext::EventWorldExt::init_events(world);
    }
}

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        Event, EventPlugin, EventQueue, EventWorldExt, Observer, ObserverId, Position, Target,
        cell_color,
    };
}
