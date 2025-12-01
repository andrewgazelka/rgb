//! World extension for event system.

use core::any::TypeId;
use std::sync::Arc;

use hashbrown::HashMap;
use parking_lot::RwLock;
use rgb_ecs::{Entity, World};
use rgb_spatial::Color;

use crate::Event;
use crate::color::cell_color;
use crate::observer::{ObserverBuilder, ObserverId, ObserverInfo};
use crate::queue::{EventQueue, QueuedEvent};

/// Target component - marks which entity an event is targeting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target(pub Entity);

/// Position component for events (determines RGB scheduling).
#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    /// Create a new position.
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Get the cell color for this position.
    #[must_use]
    pub fn color(&self) -> Color {
        cell_color(self.x, self.z)
    }
}

/// Inner event system state (behind Arc<RwLock>).
#[derive(Default)]
struct EventSystemInner {
    /// Event queue with RGB bucketing
    queue: EventQueue,
    /// Registered observers by event type
    observers: HashMap<TypeId, Vec<ObserverInfo>>,
    /// Next observer ID
    next_observer_id: u32,
}

/// Event system handle - cloneable wrapper around shared state.
///
/// This is stored as a component on Entity::WORLD.
/// The Arc<RwLock> allows it to be Clone while still holding observers.
#[derive(Clone, Default)]
pub struct EventSystem {
    inner: Arc<RwLock<EventSystemInner>>,
}

impl EventSystem {
    /// Create a new event system.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an observer for an event type.
    pub fn add_observer(&self, mut info: ObserverInfo) -> ObserverId {
        let mut inner = self.inner.write();
        let id = ObserverId::new(inner.next_observer_id);
        inner.next_observer_id += 1;
        info.id = id;

        inner
            .observers
            .entry(info.event_type_id)
            .or_default()
            .push(info);

        id
    }

    /// Push a global event to the queue.
    pub fn push_global(&self, event: QueuedEvent) {
        self.inner.write().queue.push_global(event);
    }

    /// Push a colored event to the queue.
    pub fn push_colored(&self, event: QueuedEvent, color: Color) {
        self.inner.write().queue.push_colored(event, color);
    }

    /// Pop a global event from the queue.
    pub fn pop_global(&self) -> Option<QueuedEvent> {
        self.inner.write().queue.pop_global()
    }

    /// Pop a colored event from the queue.
    pub fn pop_colored(&self, color: Color) -> Option<QueuedEvent> {
        self.inner.write().queue.pop_colored(color)
    }

    /// Check if the global queue is empty.
    #[must_use]
    pub fn is_global_empty(&self) -> bool {
        self.inner.read().queue.is_global_empty()
    }

    /// Get the number of global events.
    #[must_use]
    pub fn global_len(&self) -> usize {
        self.inner.read().queue.global_len()
    }

    /// Get the number of events for a color.
    #[must_use]
    pub fn color_len(&self, color: Color) -> usize {
        self.inner.read().queue.color_len(color)
    }
}

/// Extension trait for World to add event functionality.
pub trait EventWorldExt {
    /// Initialize the event system on this world.
    fn init_events(&mut self);

    /// Send an event to a target entity.
    ///
    /// - If target is `Entity::WORLD`, the event is global (sequential).
    /// - Otherwise, the event is scheduled by the target's position color (RGB parallel).
    fn send<E: Event + Clone>(&mut self, target: Entity, event: E);

    /// Send an event at a specific position (no target entity).
    ///
    /// The position determines the RGB color for parallel scheduling.
    /// Use this for positional events like explosions.
    fn send_at<E: Event + Clone>(&mut self, pos: Position, event: E);

    /// Register an observer for an event type.
    ///
    /// The observer callback receives:
    /// - `world`: mutable reference to the World
    /// - `target`: the target entity (or Entity::WORLD for global/positional events)
    /// - `event`: reference to the event data
    ///
    /// Returns an `ObserverId` that can be used to remove the observer later.
    fn observe<E, F>(&mut self, callback: F) -> ObserverId
    where
        E: Event,
        F: Fn(&mut World, Entity, &E) + Send + Sync + 'static;

    /// Flush all events, processing them in the correct order:
    ///
    /// ```text
    /// 1. Global events (sequential)
    /// 2. Red cell events (parallel within color)
    /// 3. Green cell events (parallel within color)
    /// 4. Blue cell events (parallel within color)
    /// 5. Global events again (for any globals queued during RGB phases)
    /// ```
    fn flush_events(&mut self);

    /// Get the event system handle.
    fn events(&self) -> Option<EventSystem>;
}

impl EventWorldExt for World {
    fn init_events(&mut self) {
        if self.get::<EventSystem>(Entity::WORLD).is_none() {
            self.insert(Entity::WORLD, EventSystem::new());
        }
    }

    fn send<E: Event + Clone>(&mut self, target: Entity, event: E) {
        self.init_events();

        // Create event entity with the event data as a component
        let event_entity = self.spawn(event);
        self.insert(event_entity, Target(target));

        let sys = self.get::<EventSystem>(Entity::WORLD).unwrap();

        let queued = QueuedEvent {
            event_entity,
            target,
            event_type_id: TypeId::of::<E>(),
        };

        if target == Entity::WORLD {
            // Global event - sequential processing
            sys.push_global(queued);
        } else {
            // Targeted event - get target's position for RGB scheduling
            let color = self
                .get::<Position>(target)
                .map_or(Color::Red, |p| p.color());

            sys.push_colored(queued, color);
        }
    }

    fn send_at<E: Event + Clone>(&mut self, pos: Position, event: E) {
        self.init_events();

        // Create event entity with position
        let event_entity = self.spawn(event);
        self.insert(event_entity, pos);
        self.insert(event_entity, Target(Entity::WORLD));

        let sys = self.get::<EventSystem>(Entity::WORLD).unwrap();

        let queued = QueuedEvent {
            event_entity,
            target: Entity::WORLD,
            event_type_id: TypeId::of::<E>(),
        };

        sys.push_colored(queued, pos.color());
    }

    fn observe<E, F>(&mut self, callback: F) -> ObserverId
    where
        E: Event,
        F: Fn(&mut World, Entity, &E) + Send + Sync + 'static,
    {
        self.init_events();

        let info = ObserverBuilder::<E>::new().build(callback);
        let sys = self.get::<EventSystem>(Entity::WORLD).unwrap();
        sys.add_observer(info)
    }

    fn flush_events(&mut self) {
        // Phase 1: Process global events
        flush_global_events(self);

        // Phase 2-4: Process RGB events in order (R → G → B)
        for color in Color::ALL {
            flush_color_events(self, color);
        }

        // Phase 5: Process any new global events added during RGB phases
        flush_global_events(self);
    }

    fn events(&self) -> Option<EventSystem> {
        self.get::<EventSystem>(Entity::WORLD)
    }
}

/// Flush all global events.
fn flush_global_events(world: &mut World) {
    loop {
        let Some(sys) = world.get::<EventSystem>(Entity::WORLD) else {
            return;
        };

        let Some(queued) = sys.pop_global() else {
            return;
        };

        process_event(world, queued);
    }
}

/// Flush all events for a specific color.
fn flush_color_events(world: &mut World, color: Color) {
    loop {
        let Some(sys) = world.get::<EventSystem>(Entity::WORLD) else {
            return;
        };

        let Some(queued) = sys.pop_colored(color) else {
            return;
        };

        process_event(world, queued);
    }
}

/// Process a single event: call all observers, then despawn the event entity.
fn process_event(world: &mut World, queued: QueuedEvent) {
    let QueuedEvent {
        event_entity,
        target,
        event_type_id,
    } = queued;

    // Get the event system to access observers
    let Some(sys) = world.get::<EventSystem>(Entity::WORLD) else {
        return;
    };

    // Call each observer for this event type
    // We need to access the inner to iterate observers
    let inner = sys.inner.read();
    if let Some(observers) = inner.observers.get(&event_type_id) {
        for observer in observers {
            // Get raw pointer to event data on the event entity
            // The observer callback will cast it to the correct type
            if let Some(event_ptr) = world.get_raw_ptr(event_entity, event_type_id) {
                // SAFETY: event_ptr is valid for the duration of this call,
                // and the callback expects the correct type
                (observer.callback)(world, target, event_ptr);
            }
        }
    }
    drop(inner);

    // Clean up event entity
    world.despawn(event_entity);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestEvent {
        value: i32,
    }

    #[test]
    fn test_init_events() {
        let mut world = World::new();
        world.init_events();

        assert!(world.events().is_some());
    }

    #[test]
    fn test_send_global_event() {
        let mut world = World::new();
        world.init_events();

        world.send(Entity::WORLD, TestEvent { value: 42 });

        let sys = world.events().unwrap();
        assert_eq!(sys.global_len(), 1);
    }

    #[test]
    fn test_send_targeted_event() {
        let mut world = World::new();
        world.init_events();

        // Create target with position
        let target = world.spawn(Position::new(0.0, 64.0, 0.0));

        world.send(target, TestEvent { value: 42 });

        let sys = world.events().unwrap();
        // Position (0, 0) -> Red cell
        assert_eq!(sys.color_len(Color::Red), 1);
    }

    #[test]
    fn test_send_at_position() {
        let mut world = World::new();
        world.init_events();

        // Position (16, 0) -> Green cell (cell 1,0 -> (1+0)%3 = 1)
        world.send_at(Position::new(16.0, 64.0, 0.0), TestEvent { value: 42 });

        let sys = world.events().unwrap();
        assert_eq!(sys.color_len(Color::Green), 1);
    }

    #[test]
    fn test_position_color() {
        assert_eq!(Position::new(0.0, 0.0, 0.0).color(), Color::Red);
        assert_eq!(Position::new(16.0, 0.0, 0.0).color(), Color::Green);
        assert_eq!(Position::new(32.0, 0.0, 0.0).color(), Color::Blue);
        assert_eq!(Position::new(48.0, 0.0, 0.0).color(), Color::Red);
    }

    #[test]
    fn test_observer_called_on_flush() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicI32, Ordering};

        let mut world = World::new();
        world.init_events();

        // Track how many times observer is called and sum of values
        let call_count = Arc::new(AtomicI32::new(0));
        let value_sum = Arc::new(AtomicI32::new(0));

        let cc = Arc::clone(&call_count);
        let vs = Arc::clone(&value_sum);

        world.observe(
            move |_world: &mut World, _target: Entity, event: &TestEvent| {
                cc.fetch_add(1, Ordering::SeqCst);
                vs.fetch_add(event.value, Ordering::SeqCst);
            },
        );

        // Send some events
        world.send(Entity::WORLD, TestEvent { value: 10 });
        world.send(Entity::WORLD, TestEvent { value: 20 });
        world.send(Entity::WORLD, TestEvent { value: 12 });

        // Observer should not be called yet
        assert_eq!(call_count.load(Ordering::SeqCst), 0);

        // Flush events
        world.flush_events();

        // Observer should have been called 3 times
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
        assert_eq!(value_sum.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_observer_receives_target() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU64, Ordering};

        let mut world = World::new();
        world.init_events();

        let received_target = Arc::new(AtomicU64::new(0));
        let rt = Arc::clone(&received_target);

        world.observe(
            move |_world: &mut World, target: Entity, _event: &TestEvent| {
                rt.store(target.to_bits(), Ordering::SeqCst);
            },
        );

        // Create target entity with position
        let target = world.spawn(Position::new(0.0, 64.0, 0.0));

        // Send targeted event
        world.send(target, TestEvent { value: 42 });
        world.flush_events();

        // Observer should have received the correct target
        assert_eq!(received_target.load(Ordering::SeqCst), target.to_bits());
    }

    #[test]
    fn test_rgb_phase_ordering() {
        use std::sync::{Arc, Mutex};

        let mut world = World::new();
        world.init_events();

        // Track order of colors processed
        let order = Arc::new(Mutex::new(Vec::new()));

        let o = Arc::clone(&order);
        world.observe(
            move |world: &mut World, target: Entity, _event: &TestEvent| {
                // Get the position to determine color
                if let Some(pos) = world.get::<Position>(target) {
                    o.lock().unwrap().push(pos.color());
                }
            },
        );

        // Create targets in different cells
        let red_target = world.spawn(Position::new(0.0, 64.0, 0.0)); // Red
        let green_target = world.spawn(Position::new(16.0, 64.0, 0.0)); // Green
        let blue_target = world.spawn(Position::new(32.0, 64.0, 0.0)); // Blue

        // Send events in reverse order (Blue, Green, Red)
        world.send(blue_target, TestEvent { value: 3 });
        world.send(green_target, TestEvent { value: 2 });
        world.send(red_target, TestEvent { value: 1 });

        world.flush_events();

        // Events should be processed in R -> G -> B order
        let processed = order.lock().unwrap();
        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0], Color::Red);
        assert_eq!(processed[1], Color::Green);
        assert_eq!(processed[2], Color::Blue);
    }
}
