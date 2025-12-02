//! Component change history tracking with persistent storage.
//!
//! Uses nebari's versioned B+tree for persistent history with time-travel.

use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use nebari::tree::{Root, Versioned};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Source of a component change.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeSource {
    /// Changed via dashboard API.
    Dashboard,
    /// Changed by game system.
    System,
    /// Initial value when entity was spawned.
    Spawn,
    /// Reverted from history.
    Revert,
}

/// A single history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique ID for this entry (sequence number from nebari).
    pub id: u64,
    /// Unix timestamp in milliseconds.
    pub timestamp: u64,
    /// Entity ID.
    pub entity: u64,
    /// Component name.
    pub component: String,
    /// Value before the change (None if added).
    pub old_value: Option<serde_json::Value>,
    /// Value after the change (None if removed).
    pub new_value: Option<serde_json::Value>,
    /// Source of the change.
    pub source: ChangeSource,
}

/// Persistent history storage using nebari.
#[derive(Clone)]
pub struct HistoryStore {
    inner: Arc<HistoryStoreInner>,
}

struct HistoryStoreInner {
    roots: nebari::Roots<nebari::io::fs::StdFile>,
    next_id: RwLock<u64>,
}

impl std::fmt::Debug for HistoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryStore").finish_non_exhaustive()
    }
}

impl Default for HistoryStore {
    fn default() -> Self {
        // Use temp directory for default
        let temp_dir = std::env::temp_dir().join("rgb-ecs-history");
        Self::open(&temp_dir).expect("Failed to open default history store")
    }
}

impl HistoryStore {
    /// Open or create a history store at the given path.
    pub fn open(path: &Path) -> Result<Self, nebari::Error> {
        std::fs::create_dir_all(path).ok();
        let config = nebari::Config::default_for(path);
        let roots = config.open()?;

        // Get the current max ID
        let tree = roots.tree(Versioned::tree("history"))?;
        let next_id = tree
            .get(b"__next_id__")?
            .map(|bytes| {
                let arr: [u8; 8] = bytes.as_ref().try_into().unwrap_or([0; 8]);
                u64::from_le_bytes(arr)
            })
            .unwrap_or(1);

        Ok(Self {
            inner: Arc::new(HistoryStoreInner {
                roots,
                next_id: RwLock::new(next_id),
            }),
        })
    }

    /// Get the history tree.
    fn tree(&self) -> Result<nebari::Tree<Versioned, nebari::io::fs::StdFile>, nebari::Error> {
        self.inner.roots.tree(Versioned::tree("history"))
    }

    /// Record a component change.
    pub fn record(
        &self,
        entity: u64,
        component: String,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        source: ChangeSource,
    ) -> u64 {
        let Ok(tree) = self.tree() else {
            return 0;
        };

        let id = {
            let mut next_id = self.inner.next_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let entry = HistoryEntry {
            id,
            timestamp,
            entity,
            component: component.clone(),
            old_value,
            new_value,
            source,
        };

        // Serialize entry
        let Ok(entry_bytes) = serde_json::to_vec(&entry) else {
            return 0;
        };

        // Key format: entity:component:id (zero-padded for sorting)
        let key = format!("{entity:016x}:{component}:{id:016x}");

        // Store the entry
        let _ = tree.set(key.into_bytes(), entry_bytes);

        // Also store by ID for direct lookup: __id__:id -> key
        let id_key = format!("__id__:{id:016x}");
        let entry_key = format!("{entity:016x}:{component}:{id:016x}");
        let _ = tree.set(id_key.into_bytes(), entry_key.into_bytes());

        // Update next_id
        let _ = tree.set(
            b"__next_id__".to_vec(),
            (*self.inner.next_id.read()).to_le_bytes().to_vec(),
        );

        id
    }

    /// Get history for a specific entity and component.
    pub fn get_component_history(
        &self,
        entity: u64,
        component: &str,
        limit: Option<usize>,
    ) -> Vec<HistoryEntry> {
        let Ok(tree) = self.tree() else {
            return Vec::new();
        };

        let prefix = format!("{entity:016x}:{component}:");
        let start = prefix.as_bytes();
        // End is prefix with last char incremented
        let mut end_bytes = prefix.clone().into_bytes();
        if let Some(last) = end_bytes.last_mut() {
            *last = last.saturating_add(1);
        }

        let limit = limit.unwrap_or(100);

        let Ok(results) = tree.get_range(&(start..&end_bytes[..])) else {
            return Vec::new();
        };

        let mut entries: Vec<HistoryEntry> = results
            .into_iter()
            .filter_map(|(_key, value)| serde_json::from_slice(&value).ok())
            .collect();

        // Sort by ID descending (most recent first)
        entries.sort_by(|a, b| b.id.cmp(&a.id));
        entries.truncate(limit);
        entries
    }

    /// Get history for a specific entity (all components).
    pub fn get_entity_history(&self, entity: u64, limit: Option<usize>) -> Vec<HistoryEntry> {
        let Ok(tree) = self.tree() else {
            return Vec::new();
        };

        let prefix = format!("{entity:016x}:");
        let start = prefix.as_bytes();
        let mut end_bytes = prefix.clone().into_bytes();
        if let Some(last) = end_bytes.last_mut() {
            *last = last.saturating_add(1);
        }

        let limit = limit.unwrap_or(100);

        let Ok(results) = tree.get_range(&(start..&end_bytes[..])) else {
            return Vec::new();
        };

        let mut entries: Vec<HistoryEntry> = results
            .into_iter()
            .filter_map(|(_key, value)| serde_json::from_slice(&value).ok())
            .collect();

        entries.sort_by(|a, b| b.id.cmp(&a.id));
        entries.truncate(limit);
        entries
    }

    /// Get global history (all entities, all components).
    pub fn get_global_history(&self, limit: Option<usize>) -> Vec<HistoryEntry> {
        let Ok(tree) = self.tree() else {
            return Vec::new();
        };

        let limit = limit.unwrap_or(100);

        // Get all non-metadata keys (those that start with a hex digit)
        let start = b"0".as_slice();
        let end = b"g".as_slice(); // After all hex digits

        let Ok(results) = tree.get_range(&(start..end)) else {
            return Vec::new();
        };

        let mut entries: Vec<HistoryEntry> = results
            .into_iter()
            .filter_map(|(_key, value)| serde_json::from_slice(&value).ok())
            .collect();

        entries.sort_by(|a, b| b.id.cmp(&a.id));
        entries.truncate(limit);
        entries
    }

    /// Get a specific entry by ID.
    pub fn get_entry(&self, id: u64) -> Option<HistoryEntry> {
        let tree = self.tree().ok()?;

        // Look up the key via the ID index
        let id_key = format!("__id__:{id:016x}");
        let entry_key = tree.get(id_key.as_bytes()).ok()??;

        // Now get the actual entry
        let entry_bytes = tree.get(&entry_key).ok()??;
        serde_json::from_slice(&entry_bytes).ok()
    }

    /// Get the value at a specific point in time for an entity-component pair.
    pub fn get_value_at_time(
        &self,
        entity: u64,
        component: &str,
        timestamp: u64,
    ) -> Option<serde_json::Value> {
        let entries = self.get_component_history(entity, component, None);

        // Find the last entry at or before the timestamp
        entries
            .into_iter()
            .filter(|e| e.timestamp <= timestamp)
            .max_by_key(|e| e.timestamp)
            .and_then(|e| e.new_value)
    }

    /// Get total entry count (approximate, for display).
    pub fn len(&self) -> usize {
        self.get_global_history(Some(100000)).len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_retrieve() {
        let dir = tempfile::tempdir().unwrap();
        let store = HistoryStore::open(dir.path()).unwrap();

        store.record(
            1,
            "Position".to_string(),
            None,
            Some(serde_json::json!({"x": 0, "y": 0})),
            ChangeSource::Spawn,
        );

        store.record(
            1,
            "Position".to_string(),
            Some(serde_json::json!({"x": 0, "y": 0})),
            Some(serde_json::json!({"x": 10, "y": 20})),
            ChangeSource::System,
        );

        let history = store.get_component_history(1, "Position", None);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].source, ChangeSource::System); // Most recent first
        assert_eq!(history[1].source, ChangeSource::Spawn);
    }

    #[test]
    fn test_entity_history() {
        let dir = tempfile::tempdir().unwrap();
        let store = HistoryStore::open(dir.path()).unwrap();

        store.record(
            1,
            "Position".to_string(),
            None,
            Some(serde_json::json!({"x": 0})),
            ChangeSource::Spawn,
        );

        store.record(
            1,
            "Health".to_string(),
            None,
            Some(serde_json::json!({"hp": 100})),
            ChangeSource::Spawn,
        );

        let history = store.get_entity_history(1, None);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_get_entry_by_id() {
        let dir = tempfile::tempdir().unwrap();
        let store = HistoryStore::open(dir.path()).unwrap();

        let id = store.record(
            42,
            "Health".to_string(),
            None,
            Some(serde_json::json!({"hp": 100})),
            ChangeSource::Dashboard,
        );

        let entry = store.get_entry(id).unwrap();
        assert_eq!(entry.entity, 42);
        assert_eq!(entry.component, "Health");
        assert_eq!(entry.source, ChangeSource::Dashboard);
    }
}
