//! Component history tracking system
//!
//! This module provides in-memory history tracking for components.
//! For now, this is a placeholder - full history will be implemented with Flecs observers.

use flecs_ecs::prelude::*;

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

/// Component that tracks history for a specific component type
#[derive(Component, Debug, Clone, Default)]
pub struct ComponentHistory {
    /// Name of the component type being tracked
    pub component_name: String,
    /// History entries (most recent last)
    pub entries: Vec<HistoryEntry>,
    /// Maximum entries to keep
    pub max_entries: usize,
}

impl ComponentHistory {
    pub fn new(component_name: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
            entries: Vec::new(),
            max_entries: 1000,
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
            self.entries.remove(0);
        }

        self.entries.push(HistoryEntry {
            tick,
            old_value,
            new_value,
            change_type,
        });
    }

    /// Get the last N entries (most recent first)
    pub fn last_n(&self, n: usize) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.iter().rev().take(n)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Format history entries for display
pub fn format_history(history: &ComponentHistory, max_entries: usize) -> String {
    let mut lines = Vec::new();
    lines.push(format!("History for {}:", history.component_name));

    for entry in history.last_n(max_entries) {
        let change_str = match entry.change_type {
            ChangeType::Added => "+",
            ChangeType::Modified => "~",
            ChangeType::Removed => "-",
        };

        let value_str = match (&entry.old_value, entry.change_type) {
            (Some(old), ChangeType::Modified) => {
                format!("{} -> {}", old, entry.new_value)
            }
            (_, ChangeType::Removed) => entry.old_value.clone().unwrap_or_default(),
            _ => entry.new_value.clone(),
        };

        lines.push(format!(
            "  [tick {}] {} {}",
            entry.tick, change_str, value_str
        ));
    }

    if lines.len() == 1 {
        lines.push("  (no history recorded)".to_string());
    }

    lines.join("\n")
}
