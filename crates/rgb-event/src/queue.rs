//! Event queue with RGB bucketing.

use std::any::TypeId;
use std::collections::VecDeque;

use rgb_ecs::Entity;
use rgb_spatial::Color;

/// A queued event waiting to be processed.
pub struct QueuedEvent {
    /// The event entity (contains event data as component)
    pub event_entity: Entity,
    /// Target entity for this event
    pub target: Entity,
    /// Event component type
    pub event_type_id: TypeId,
}

impl core::fmt::Debug for QueuedEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QueuedEvent")
            .field("event_entity", &self.event_entity)
            .field("target", &self.target)
            .finish()
    }
}

/// Event queue with separate buckets for global and RGB-colored events.
///
/// Events are bucketed by the cell color of their target's position:
/// - Global events (target = Entity::WORLD) → global queue
/// - Targeted events → RGB queue by target's position color
#[derive(Default)]
pub struct EventQueue {
    /// Global events (no position, sequential processing)
    global: VecDeque<QueuedEvent>,
    /// Red cell events (parallel with other red cells)
    red: VecDeque<QueuedEvent>,
    /// Green cell events (parallel with other green cells)
    green: VecDeque<QueuedEvent>,
    /// Blue cell events (parallel with other blue cells)
    blue: VecDeque<QueuedEvent>,
}

impl EventQueue {
    /// Create a new empty event queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a global event (target = Entity::WORLD).
    pub fn push_global(&mut self, event: QueuedEvent) {
        debug_assert!(
            event.target == Entity::WORLD,
            "Global events must target Entity::WORLD"
        );
        self.global.push_back(event);
    }

    /// Push a colored/spatial event with explicit color.
    ///
    /// This is used for:
    /// - Targeted events (target entity has a position)
    /// - Positional events (`send_at`) where event has position but targets WORLD
    pub fn push_colored(&mut self, event: QueuedEvent, color: Color) {
        match color {
            Color::Red => self.red.push_back(event),
            Color::Green => self.green.push_back(event),
            Color::Blue => self.blue.push_back(event),
        }
    }

    /// Pop the next global event.
    pub fn pop_global(&mut self) -> Option<QueuedEvent> {
        self.global.pop_front()
    }

    /// Pop the next event for a specific color.
    pub fn pop_colored(&mut self, color: Color) -> Option<QueuedEvent> {
        match color {
            Color::Red => self.red.pop_front(),
            Color::Green => self.green.pop_front(),
            Color::Blue => self.blue.pop_front(),
        }
    }

    /// Drain all events for a specific color.
    pub fn drain_colored(&mut self, color: Color) -> impl Iterator<Item = QueuedEvent> + '_ {
        match color {
            Color::Red => self.red.drain(..),
            Color::Green => self.green.drain(..),
            Color::Blue => self.blue.drain(..),
        }
    }

    /// Drain all global events.
    pub fn drain_global(&mut self) -> impl Iterator<Item = QueuedEvent> + '_ {
        self.global.drain(..)
    }

    /// Check if the global queue is empty.
    #[must_use]
    pub fn is_global_empty(&self) -> bool {
        self.global.is_empty()
    }

    /// Check if a color queue is empty.
    #[must_use]
    pub fn is_color_empty(&self, color: Color) -> bool {
        match color {
            Color::Red => self.red.is_empty(),
            Color::Green => self.green.is_empty(),
            Color::Blue => self.blue.is_empty(),
        }
    }

    /// Check if all queues are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.global.is_empty()
            && self.red.is_empty()
            && self.green.is_empty()
            && self.blue.is_empty()
    }

    /// Get total number of queued events.
    #[must_use]
    pub fn len(&self) -> usize {
        self.global.len() + self.red.len() + self.green.len() + self.blue.len()
    }

    /// Get number of global events.
    #[must_use]
    pub fn global_len(&self) -> usize {
        self.global.len()
    }

    /// Get number of events for a specific color.
    #[must_use]
    pub fn color_len(&self, color: Color) -> usize {
        match color {
            Color::Red => self.red.len(),
            Color::Green => self.green.len(),
            Color::Blue => self.blue.len(),
        }
    }

    /// Clear all queues.
    pub fn clear(&mut self) {
        self.global.clear();
        self.red.clear();
        self.green.clear();
        self.blue.clear();
    }
}

impl core::fmt::Debug for EventQueue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EventQueue")
            .field("global", &self.global.len())
            .field("red", &self.red.len())
            .field("green", &self.green.len())
            .field("blue", &self.blue.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_event(target: Entity) -> QueuedEvent {
        QueuedEvent {
            event_entity: Entity::from_bits(100),
            target,
            event_type_id: TypeId::of::<()>(),
        }
    }

    #[test]
    fn test_global_queue() {
        let mut queue = EventQueue::new();
        assert!(queue.is_empty());

        queue.push_global(dummy_event(Entity::WORLD));
        assert!(!queue.is_global_empty());
        assert_eq!(queue.global_len(), 1);

        let event = queue.pop_global().unwrap();
        assert_eq!(event.target, Entity::WORLD);
        assert!(queue.is_global_empty());
    }

    #[test]
    fn test_colored_queues() {
        let mut queue = EventQueue::new();

        let target = Entity::from_bits(1);
        queue.push_colored(dummy_event(target), Color::Red);
        queue.push_colored(dummy_event(target), Color::Green);
        queue.push_colored(dummy_event(target), Color::Blue);

        assert_eq!(queue.color_len(Color::Red), 1);
        assert_eq!(queue.color_len(Color::Green), 1);
        assert_eq!(queue.color_len(Color::Blue), 1);
        assert_eq!(queue.len(), 3);

        assert!(queue.pop_colored(Color::Red).is_some());
        assert!(queue.is_color_empty(Color::Red));
    }

    #[test]
    fn test_drain() {
        let mut queue = EventQueue::new();
        let target = Entity::from_bits(1);

        for _ in 0..5 {
            queue.push_colored(dummy_event(target), Color::Red);
        }

        let drained: Vec<_> = queue.drain_colored(Color::Red).collect();
        assert_eq!(drained.len(), 5);
        assert!(queue.is_color_empty(Color::Red));
    }
}
