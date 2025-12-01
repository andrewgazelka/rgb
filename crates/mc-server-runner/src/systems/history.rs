//! Component history tracking system
//!
//! This module provides in-memory history tracking for components.
//!
//! For persistent history with time-travel, use `rgb_storage::VersionedWorld`
//! which stores every mutation in Nebari's versioned B+tree, allowing:
//! - `get_at_tick::<T>(entity, tick)` - read any component at any historical tick
//! - `revert_to_tick(tick)` - jump back to any previous state
//!
//! This in-memory history is useful for:
//! - Quick debugging during development
//! - Display in the /history command
//! - Cases where persistence isn't needed

use std::any::TypeId;
use std::collections::BTreeMap;

use rgb_ecs::{Entity, World};

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
    Added,
    Modified,
    Removed,
}

/// Component that tracks history for a specific component type using BTreeMap
/// for efficient tick-based lookups and range queries.
#[derive(Debug, Clone)]
pub struct ComponentHistory {
    /// Name of the component type being tracked
    pub component_name: String,
    /// TypeId of the component (for type safety)
    pub type_id: TypeId,
    /// BTree of tick -> history entry for O(log n) lookups
    pub entries: BTreeMap<i64, HistoryEntry>,
    /// Maximum entries to keep (oldest removed when exceeded)
    pub max_entries: usize,
}

impl Default for ComponentHistory {
    fn default() -> Self {
        Self {
            component_name: String::new(),
            type_id: TypeId::of::<()>(),
            entries: BTreeMap::new(),
            max_entries: 1000,
        }
    }
}

impl ComponentHistory {
    pub fn new<T: 'static>(component_name: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
            type_id: TypeId::of::<T>(),
            entries: BTreeMap::new(),
            max_entries: 1000,
        }
    }

    pub fn with_max_entries<T: 'static>(component_name: &str, max_entries: usize) -> Self {
        Self {
            component_name: component_name.to_string(),
            type_id: TypeId::of::<T>(),
            entries: BTreeMap::new(),
            max_entries,
        }
    }

    pub fn record(
        &mut self,
        tick: i64,
        old_value: Option<String>,
        new_value: String,
        change_type: ChangeType,
    ) {
        // Trim oldest entries if at capacity
        while self.entries.len() >= self.max_entries {
            if let Some(oldest_tick) = self.entries.keys().next().copied() {
                self.entries.remove(&oldest_tick);
            }
        }

        self.entries.insert(
            tick,
            HistoryEntry {
                tick,
                old_value,
                new_value,
                change_type,
            },
        );
    }

    /// Get entry at exact tick
    pub fn at_tick(&self, tick: i64) -> Option<&HistoryEntry> {
        self.entries.get(&tick)
    }

    /// Get the most recent entry at or before the given tick
    pub fn at_or_before_tick(&self, tick: i64) -> Option<&HistoryEntry> {
        self.entries.range(..=tick).next_back().map(|(_, e)| e)
    }

    /// Get entries within a tick range (inclusive)
    pub fn in_range(&self, start_tick: i64, end_tick: i64) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.range(start_tick..=end_tick).map(|(_, e)| e)
    }

    /// Get the last N entries (most recent first)
    pub fn last_n(&self, n: usize) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.values().rev().take(n)
    }

    /// Get all entries in chronological order
    pub fn all(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.values()
    }

    /// Number of recorded entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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

    pub fn enable(&mut self, entity: Entity, component_name: &str, history_entity: Entity) {
        self.tracked
            .insert((entity.id(), component_name.to_string()), history_entity);
    }

    pub fn disable(&mut self, entity: Entity, component_name: &str) {
        self.tracked
            .remove(&(entity.id(), component_name.to_string()));
    }

    pub fn get(&self, entity: Entity, component_name: &str) -> Option<Entity> {
        self.tracked
            .get(&(entity.id(), component_name.to_string()))
            .copied()
    }

    pub fn is_tracked(&self, entity: Entity, component_name: &str) -> bool {
        self.tracked
            .contains_key(&(entity.id(), component_name.to_string()))
    }
}

/// Helper trait for recording component changes
pub trait RecordHistory {
    fn to_history_string(&self) -> String;
}

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
    let Some(registry) = world.get::<HistoryRegistry>(Entity::WORLD) else {
        return;
    };

    let Some(history_entity) = registry.get(entity, "Position") else {
        return;
    };

    let Some(mut history) = world.get::<ComponentHistory>(history_entity) else {
        return;
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

/// Enable rotation history tracking for an entity
pub fn enable_rotation_history(world: &mut World, entity: Entity) {
    let history_entity = world.spawn(ComponentHistory::new::<crate::components::Rotation>(
        "Rotation",
    ));

    let mut registry = world
        .get::<HistoryRegistry>(Entity::WORLD)
        .unwrap_or_default();
    registry.enable(entity, "Rotation", history_entity);
    world.update(Entity::WORLD, registry);
}

/// Enable name history tracking for an entity
pub fn enable_name_history(world: &mut World, entity: Entity) {
    let history_entity = world.spawn(ComponentHistory::new::<crate::components::Name>("Name"));

    let mut registry = world
        .get::<HistoryRegistry>(Entity::WORLD)
        .unwrap_or_default();
    registry.enable(entity, "Name", history_entity);
    world.update(Entity::WORLD, registry);
}

/// Enable history tracking for a component by name
pub fn enable_history_by_name(
    world: &mut World,
    entity: Entity,
    component_name: &str,
) -> Result<(), String> {
    match component_name {
        "Position" => {
            enable_position_history(world, entity);
            Ok(())
        }
        "Rotation" => {
            enable_rotation_history(world, entity);
            Ok(())
        }
        "Name" => {
            enable_name_history(world, entity);
            Ok(())
        }
        _ => Err(format!("Unknown component: {}", component_name)),
    }
}
