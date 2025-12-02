//! Component persistence using Nebari database.
//!
//! Stores component history with keys: `{uuid}:{component_name}:{timestamp}`
//! This enables querying history for any entity by UUID.

#![allow(unsafe_code)]

use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crossbeam_channel::{Receiver, Sender, bounded};
use flecs_ecs::prelude::*;
use flecs_history::prelude::*;
use nebari::tree::{Root, ScanEvaluation, Unversioned};
use nebari::{Config, Error as NebariError};
use serde::{Deserialize, Serialize};

use crate::components::Uuid;

/// History entry stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub timestamp: u64,
    pub entity_uuid: u128,
    pub entity_id: u64,
    pub component: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub source: ChangeSource,
}

/// Source of a component change.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChangeSource {
    #[default]
    System,
    Dashboard,
    Spawn,
    Revert,
}

/// Request to persist a component change.
pub struct PersistRequest {
    pub entity_uuid: u128,
    pub entity_id: u64,
    pub component_name: String,
    pub new_value: serde_json::Value,
    pub serialized_bytes: Vec<u8>,
    pub source: ChangeSource,
}

/// Channels for persistence communication.
pub struct PersistenceChannels {
    pub tx: Sender<PersistRequest>,
    pub rx: Receiver<PersistRequest>,
}

impl PersistenceChannels {
    pub fn new() -> Self {
        let (tx, rx) = bounded(1024);
        Self { tx, rx }
    }
}

impl Default for PersistenceChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to the persistence database.
#[derive(Clone)]
pub struct PersistenceDb {
    roots: nebari::Roots<nebari::io::fs::StdFile>,
    /// Counter for entry IDs.
    next_id: Arc<std::sync::atomic::AtomicU64>,
}

impl PersistenceDb {
    /// Open or create the persistence database.
    pub fn open(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let roots = Config::default_for(path.as_ref())
            .open()
            .map_err(|e| eyre::eyre!("Failed to open nebari database: {e}"))?;

        // Start with ID 1 (we don't scan for max ID to keep it simple)
        Ok(Self {
            roots,
            next_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        })
    }

    /// Get or create the history tree.
    fn history_tree(
        &self,
    ) -> Result<nebari::Tree<Unversioned, nebari::io::fs::StdFile>, NebariError> {
        self.roots.tree(Unversioned::tree("history"))
    }

    /// Get or create the current value tree.
    fn current_tree(
        &self,
    ) -> Result<nebari::Tree<Unversioned, nebari::io::fs::StdFile>, NebariError> {
        self.roots.tree(Unversioned::tree("current"))
    }

    /// Store a component change in history and update current value.
    pub fn persist(&self, request: PersistRequest) -> eyre::Result<HistoryEntry> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Get old value from current tree
        let current_tree = self
            .current_tree()
            .map_err(|e| eyre::eyre!("Failed to open current tree: {e}"))?;
        let current_key = format!("{}:{}", request.entity_uuid, request.component_name);
        let old_value = current_tree
            .get(current_key.as_bytes())
            .map_err(|e| eyre::eyre!("Failed to get current value: {e}"))?
            .and_then(|bytes| serde_json::from_slice(&bytes).ok());

        let entry = HistoryEntry {
            id,
            timestamp,
            entity_uuid: request.entity_uuid,
            entity_id: request.entity_id,
            component: request.component_name.clone(),
            old_value,
            new_value: request.new_value.clone(),
            source: request.source,
        };

        // Store history entry
        // Key format: {uuid}:{timestamp}:{id} for chronological ordering
        let history_key = format!("{:032x}:{:016x}:{:016x}", entry.entity_uuid, timestamp, id);
        let history_tree = self
            .history_tree()
            .map_err(|e| eyre::eyre!("Failed to open history tree: {e}"))?;
        let entry_bytes = bincode::serialize(&entry)
            .map_err(|e| eyre::eyre!("Failed to serialize entry: {e}"))?;
        history_tree
            .set(history_key, entry_bytes)
            .map_err(|e| eyre::eyre!("Failed to store history entry: {e}"))?;

        // Update current value
        let new_value_bytes = serde_json::to_vec(&request.new_value)
            .map_err(|e| eyre::eyre!("Failed to serialize new value: {e}"))?;
        current_tree
            .set(current_key, new_value_bytes)
            .map_err(|e| eyre::eyre!("Failed to store current value: {e}"))?;

        Ok(entry)
    }

    /// Get history for a specific entity by UUID.
    pub fn get_entity_history(&self, uuid: u128, limit: usize) -> eyre::Result<Vec<HistoryEntry>> {
        let history_tree = self
            .history_tree()
            .map_err(|e| eyre::eyre!("Failed to open history tree: {e}"))?;
        let prefix = format!("{:032x}:", uuid);

        let mut entries: Vec<HistoryEntry> = Vec::new();

        history_tree
            .scan::<std::convert::Infallible, _, _, _, _>(
                &(..),
                true, // forwards
                |_key, _reduced_index, _depth| ScanEvaluation::ReadData,
                |key, _index| {
                    let key_str = String::from_utf8_lossy(key);
                    if key_str.starts_with(&prefix) {
                        ScanEvaluation::ReadData
                    } else {
                        ScanEvaluation::Skip
                    }
                },
                |key, _index, value| {
                    let key_str = String::from_utf8_lossy(&key);
                    if key_str.starts_with(&prefix) {
                        if let Ok(entry) = bincode::deserialize::<HistoryEntry>(&value) {
                            entries.push(entry);
                        }
                    }
                    Ok(())
                },
            )
            .map_err(|e| eyre::eyre!("Failed to scan history: {e}"))?;

        // Sort by timestamp descending (newest first)
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(limit);

        Ok(entries)
    }

    /// Get global history (all entities).
    pub fn get_global_history(&self, limit: usize) -> eyre::Result<Vec<HistoryEntry>> {
        let history_tree = self
            .history_tree()
            .map_err(|e| eyre::eyre!("Failed to open history tree: {e}"))?;

        let mut entries: Vec<HistoryEntry> = Vec::new();

        history_tree
            .scan::<std::convert::Infallible, _, _, _, _>(
                &(..),
                true, // forwards
                |_key, _reduced_index, _depth| ScanEvaluation::ReadData,
                |_key, _index| ScanEvaluation::ReadData,
                |_key, _index, value| {
                    if let Ok(entry) = bincode::deserialize::<HistoryEntry>(&value) {
                        entries.push(entry);
                    }
                    Ok(())
                },
            )
            .map_err(|e| eyre::eyre!("Failed to scan history: {e}"))?;

        // Sort by timestamp descending (newest first)
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(limit);

        Ok(entries)
    }

    /// Get history for a specific component on an entity.
    pub fn get_component_history(
        &self,
        uuid: u128,
        component: &str,
        limit: usize,
    ) -> eyre::Result<Vec<HistoryEntry>> {
        let mut entries = self.get_entity_history(uuid, limit * 10)?; // Get more, then filter
        entries.retain(|e| e.component == component);
        entries.truncate(limit);
        Ok(entries)
    }
}

/// Register the wildcard observer for automatic persistence.
///
/// This observer fires on ANY component OnSet and persists changes for
/// entities that have a Uuid component to components that have SerializeInfo.
pub fn register_persistence_observer(world: &World, persist_tx: Sender<PersistRequest>) {
    world
        .observer::<flecs::OnSet, ()>()
        .with(flecs::Wildcard::id())
        .each_iter(move |it, index, _| {
            let entity = it.entity(index);
            let component_id = it.event_id();
            let world = it.world();
            let comp_entity = world.entity_from_id(*component_id.id());

            // Check if entity has Uuid
            let Some(uuid) = entity.try_get::<&Uuid>(|u| u.0) else {
                return; // Entity doesn't have Uuid, skip persistence
            };

            // Check if component has SerializeInfo
            let Some(info) = comp_entity.try_get::<&SerializeInfo>(|s| s.clone()) else {
                return; // Component not serializable
            };

            let comp_name = comp_entity.name();

            // JANK: Using raw ecs_get_id to get component pointer since wildcard
            // observer doesn't give us typed access. There might be a better way
            // to get the component data in a wildcard observer context.
            let comp_ptr = unsafe {
                flecs_ecs::sys::ecs_get_id(world.world_ptr(), entity.id().0, *component_id.id())
            };

            if comp_ptr.is_null() {
                return;
            }

            // Serialize using SerializeInfo
            let bytes = (info.to_bytes)(comp_ptr, info.component_size);
            let json_value = (info.to_json)(comp_ptr);

            // Send persist request
            let _ = persist_tx.try_send(PersistRequest {
                entity_uuid: uuid,
                entity_id: entity.id().0,
                component_name: comp_name,
                new_value: json_value,
                serialized_bytes: bytes,
                source: ChangeSource::System,
            });
        });
}

/// Process pending persistence requests (call from game loop).
pub fn process_persistence_requests(db: &PersistenceDb, rx: &Receiver<PersistRequest>) {
    while let Ok(request) = rx.try_recv() {
        if let Err(e) = db.persist(request) {
            tracing::warn!("Failed to persist component change: {e}");
        }
    }
}
