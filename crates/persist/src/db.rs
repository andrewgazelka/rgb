//! LMDB database wrapper for component persistence.

use std::path::Path;

use heed::{Database, Env, EnvOpenOptions, types::Bytes};

/// LMDB database wrapper for persisting components.
///
/// Uses the key format `"{uuid}.{component_name}"` for storage.
pub struct PersistDb {
    env: Env,
    db: Database<Bytes, Bytes>,
}

impl PersistDb {
    /// Open or create a persistence database at the given path.
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened or created.
    ///
    /// # Safety
    /// Uses unsafe to call heed's open method which requires ensuring
    /// the database is not opened multiple times with different options.
    #[allow(unsafe_code)]
    pub fn open(path: impl AsRef<Path>) -> heed::Result<Self> {
        let path = path.as_ref();

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::create_dir_all(path).ok();

        // SAFETY: We only open this database once in the application
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(1024 * 1024 * 1024) // 1GB max
                .max_dbs(1)
                .open(path)?
        };

        let mut wtxn = env.write_txn()?;
        let db = env.create_database(&mut wtxn, Some("components"))?;
        wtxn.commit()?;

        Ok(Self { env, db })
    }

    /// Save raw bytes for a given UUID and component name.
    ///
    /// Key format: `"{uuid}.{component_name}"`
    ///
    /// # Errors
    /// Returns an error if database write fails.
    pub fn save_bytes(&self, uuid: u128, component_name: &str, bytes: &[u8]) -> heed::Result<()> {
        let key = format_key(uuid, component_name);

        let mut wtxn = self.env.write_txn()?;
        self.db.put(&mut wtxn, key.as_bytes(), bytes)?;
        wtxn.commit()?;

        tracing::trace!("Persisted {component_name} for {uuid:032x}");
        Ok(())
    }

    /// Load raw bytes for a given UUID and component name.
    ///
    /// Returns `None` if no data exists for this UUID/component combination.
    ///
    /// # Errors
    /// Returns an error if database read fails.
    pub fn load_bytes(&self, uuid: u128, component_name: &str) -> heed::Result<Option<Vec<u8>>> {
        let key = format_key(uuid, component_name);

        let rtxn = self.env.read_txn()?;
        let Some(bytes) = self.db.get(&rtxn, key.as_bytes())? else {
            return Ok(None);
        };

        tracing::trace!("Loaded {component_name} for {uuid:032x}");
        Ok(Some(bytes.to_vec()))
    }

    /// Delete a component for a given UUID.
    ///
    /// # Errors
    /// Returns an error if database delete fails.
    pub fn delete(&self, uuid: u128, component_name: &str) -> heed::Result<bool> {
        let key = format_key(uuid, component_name);

        let mut wtxn = self.env.write_txn()?;
        let deleted = self.db.delete(&mut wtxn, key.as_bytes())?;
        wtxn.commit()?;

        if deleted {
            tracing::trace!("Deleted {component_name} for {uuid:032x}");
        }
        Ok(deleted)
    }
}

/// Format the database key as `"{uuid}.{component_name}"`.
fn format_key(uuid: u128, component_name: &str) -> String {
    let uuid = uuid::Uuid::from_u128(uuid);
    format!("{uuid}.{component_name}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestPosition {
        x: f64,
        y: f64,
        z: f64,
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let db = PersistDb::open(dir.path()).unwrap();

        let uuid = 0x550e8400_e29b_41d4_a716_446655440000u128;
        let pos = TestPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        let bytes = bincode::serialize(&pos).unwrap();
        db.save_bytes(uuid, "Position", &bytes).unwrap();

        let loaded_bytes = db.load_bytes(uuid, "Position").unwrap().unwrap();
        let loaded: TestPosition = bincode::deserialize(&loaded_bytes).unwrap();
        assert_eq!(loaded, pos);
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let db = PersistDb::open(dir.path()).unwrap();

        let uuid = 0x550e8400_e29b_41d4_a716_446655440000u128;
        let loaded = db.load_bytes(uuid, "Position").unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn test_delete() {
        let dir = tempfile::tempdir().unwrap();
        let db = PersistDb::open(dir.path()).unwrap();

        let uuid = 0x550e8400_e29b_41d4_a716_446655440000u128;
        let pos = TestPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        let bytes = bincode::serialize(&pos).unwrap();
        db.save_bytes(uuid, "Position", &bytes).unwrap();
        assert!(db.delete(uuid, "Position").unwrap());

        let loaded = db.load_bytes(uuid, "Position").unwrap();
        assert_eq!(loaded, None);
    }
}
