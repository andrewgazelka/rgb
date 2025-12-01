//! Component history tracking system
//!
//! Provides a mechanism to track changes to components over time,
//! similar to Flecs' component history queries.
//!
//! This is useful for:
//! - Debugging entity state changes
//! - Implementing undo/redo
//! - Recording replays
//! - Analyzing behavior patterns

use std::any::TypeId;
use std::collections::VecDeque;

use rgb_ecs::{Entity, World};

/// Maximum number of history entries to keep per component
pub const MAX_HISTORY_ENTRIES: usize = 100;

/// A single entry in the component history
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// World tick when this change occurred
    pub tick: i64,
    /// String representation of the old value
    pub old_value: Option<String>,
    /// String representation of the new value
    pub new_value: String,
    /// Type of change
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// Component was added
    Added,
    /// Component was modified
    Modified,
    /// Component was removed
    Removed,
}

/// Component that tracks history for a specific component type
#[derive(Debug, Clone)]
pub struct ComponentHistory {
    /// Name of the component type being tracked
    pub component_name: String,
    /// TypeId of the component (for type safety)
    pub type_id: TypeId,
    /// Ring buffer of history entries
    pub entries: VecDeque<HistoryEntry>,
}

impl ComponentHistory {
    pub fn new<T: 'static>(component_name: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
            type_id: TypeId::of::<T>(),
            entries: VecDeque::with_capacity(MAX_HISTORY_ENTRIES),
        }
    }

    pub fn record(
        &mut self,
        tick: i64,
        old_value: Option<String>,
        new_value: String,
        change_type: ChangeType,
    ) {
        if self.entries.len() >= MAX_HISTORY_ENTRIES {
            self.entries.pop_front();
        }
        self.entries.push_back(HistoryEntry {
            tick,
            old_value,
            new_value,
            change_type,
        });
    }

    /// Get the last N entries
    pub fn last_n(&self, n: usize) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.iter().rev().take(n)
    }

    /// Get entries within a tick range
    pub fn in_range(&self, start_tick: i64, end_tick: i64) -> impl Iterator<Item = &HistoryEntry> {
        self.entries
            .iter()
            .filter(move |e| e.tick >= start_tick && e.tick <= end_tick)
    }
}

/// Global history registry - tracks which entities/components have history enabled
#[derive(Default, Clone)]
pub struct HistoryRegistry {
    /// Maps (entity_id, component_name) to history component entity
    pub tracked: std::collections::HashMap<(u32, String), Entity>,
}

impl HistoryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable history tracking for a component on an entity
    pub fn enable(&mut self, entity: Entity, component_name: &str, history_entity: Entity) {
        self.tracked
            .insert((entity.id(), component_name.to_string()), history_entity);
    }

    /// Disable history tracking
    pub fn disable(&mut self, entity: Entity, component_name: &str) {
        self.tracked
            .remove(&(entity.id(), component_name.to_string()));
    }

    /// Get the history entity for a tracked component
    pub fn get(&self, entity: Entity, component_name: &str) -> Option<Entity> {
        self.tracked
            .get(&(entity.id(), component_name.to_string()))
            .copied()
    }
}

/// Helper trait for recording component changes
pub trait RecordHistory {
    fn to_history_string(&self) -> String;
}

// Implement for common types
impl RecordHistory for crate::components::Position {
    fn to_history_string(&self) -> String {
        format!("({:.2}, {:.2}, {:.2})", self.x, self.y, self.z)
    }
}

impl RecordHistory for crate::components::Rotation {
    fn to_history_string(&self) -> String {
        format!("(yaw={:.1}, pitch={:.1})", self.yaw, self.pitch)
    }
}

impl RecordHistory for crate::components::Name {
    fn to_history_string(&self) -> String {
        self.value.clone()
    }
}

impl RecordHistory for crate::components::EntityId {
    fn to_history_string(&self) -> String {
        self.value.to_string()
    }
}

/// Record a position change in history
pub fn record_position_change(
    world: &mut World,
    entity: Entity,
    old_pos: Option<&crate::components::Position>,
    new_pos: &crate::components::Position,
    tick: i64,
) {
    let registry = match world.get::<HistoryRegistry>(Entity::WORLD) {
        Some(r) => r,
        None => return,
    };

    let history_entity = match registry.get(entity, "Position") {
        Some(e) => e,
        None => return,
    };

    let mut history = match world.get::<ComponentHistory>(history_entity) {
        Some(h) => h,
        None => return,
    };

    let change_type = if old_pos.is_some() {
        ChangeType::Modified
    } else {
        ChangeType::Added
    };

    history.record(
        tick,
        old_pos.map(|p| p.to_history_string()),
        new_pos.to_history_string(),
        change_type,
    );

    world.update(history_entity, history);
}

/// Query history for a specific entity and component
pub fn query_history<'a>(
    world: &'a World,
    entity: Entity,
    component_name: &str,
) -> Option<&'a ComponentHistory> {
    let registry = world.get_ref::<HistoryRegistry>(Entity::WORLD)?;
    let history_entity = registry.get(entity, component_name)?;
    world.get_ref::<ComponentHistory>(history_entity)
}

/// Format history entries for display (e.g., in /history command)
pub fn format_history(history: &ComponentHistory, max_entries: usize) -> String {
    let mut lines = Vec::new();
    lines.push(format!("§aHistory for §f{}§a:", history.component_name));

    for entry in history.last_n(max_entries) {
        let change_str = match entry.change_type {
            ChangeType::Added => "§a+",
            ChangeType::Modified => "§e~",
            ChangeType::Removed => "§c-",
        };

        let value_str = match (&entry.old_value, entry.change_type) {
            (Some(old), ChangeType::Modified) => {
                format!("{} §7→ §f{}", old, entry.new_value)
            }
            (_, ChangeType::Removed) => entry.old_value.clone().unwrap_or_default(),
            _ => entry.new_value.clone(),
        };

        lines.push(format!(
            "  §7[tick {}] {} §f{}",
            entry.tick, change_str, value_str
        ));
    }

    if lines.len() == 1 {
        lines.push("  §7(no history recorded)".to_string());
    }

    lines.join("\n")
}

/// Enable position history tracking for an entity
pub fn enable_position_history(world: &mut World, entity: Entity) {
    let history_entity = world.spawn(ComponentHistory::new::<crate::components::Position>(
        "Position",
    ));

    let mut registry = world
        .get::<HistoryRegistry>(Entity::WORLD)
        .unwrap_or_default();
    registry.enable(entity, "Position", history_entity);
    world.update(Entity::WORLD, registry);
}
