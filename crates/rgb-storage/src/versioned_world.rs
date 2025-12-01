//! Versioned world storage with time-travel capability.
//!
//! This module provides `VersionedWorld`, which wraps the ECS world with
//! Nebari's versioned B+tree storage. Every mutation is recorded, allowing:
//!
//! - **get_at_tick**: Read any component at any historical tick
//! - **revert_to_tick**: Jump the world back to any tick
//! - **commit_tick**: Atomically commit all pending changes
//!
//! # Thread-Local Buffers
//!
//! For parallel RGB phases, use `ThreadLocalBuffers` to accumulate mutations
//! without synchronization, then call `commit_tick_from_buffers` after the
//! barrier.

use nebari::tree::Root as _;
use rgb_ecs::{Entity, World};

use crate::{
    TickId,
    buffer::{Mutation, MutationBuffers},
    error::StorageResult,
    keys::ComponentKey,
};

/// A world with versioned persistent storage.
///
/// Every tick is recorded in the database, allowing time-travel queries
/// and instant revert to any historical state.
///
/// # Parallel Usage
///
/// For RGB parallel phases:
/// 1. Create `MutationBuffers` with `MutationBuffers::new()`
/// 2. During parallel phases, push mutations via `buffers.push_set()` etc.
/// 3. After barrier, call `commit_tick_from_buffers(&mut buffers)`
pub struct VersionedWorld {
    /// The in-memory ECS world (current state).
    world: World,
    /// Nebari database roots.
    roots: nebari::Roots<nebari::io::fs::StdFile>,
    /// Current tick ID (we track this separately since nebari uses sequences).
    current_tick: TickId,
    /// Pending mutations for single-threaded usage.
    pending: Vec<Mutation>,
}

impl VersionedWorld {
    /// Create a new versioned world at the given path.
    ///
    /// If the database already exists, it will be opened and the world
    /// will be restored to the latest committed tick.
    pub fn open(path: impl AsRef<std::path::Path>) -> StorageResult<Self> {
        let config = nebari::Config::default_for(path.as_ref());
        let roots = config.open()?;

        // Get or create the component tree
        let tree = roots.tree(nebari::tree::Versioned::tree("components"))?;

        // Read the current tick from a metadata key
        let current_tick = tree
            .get(b"__tick__")?
            .map(|bytes| {
                let arr: [u8; 8] = bytes.as_ref().try_into().unwrap_or([0; 8]);
                u64::from_le_bytes(arr)
            })
            .unwrap_or(0);

        // TODO: Restore world state from the tree
        // For now, start with empty world
        let world = World::new();

        Ok(Self {
            world,
            roots,
            current_tick,
            pending: Vec::new(),
        })
    }

    /// Create a new versioned world (fails if path exists).
    pub fn create(path: impl AsRef<std::path::Path>) -> StorageResult<Self> {
        // TODO: Check if path exists and fail
        Self::open(path)
    }

    /// Get the current tick ID.
    #[must_use]
    pub fn current_tick(&self) -> TickId {
        self.current_tick
    }

    /// Get a reference to the in-memory world.
    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get a mutable reference to the in-memory world.
    ///
    /// Note: Mutations made directly to the world are NOT persisted.
    /// Use the `insert`, `update`, `remove` methods instead.
    #[must_use]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Helper to get the component tree.
    fn tree(
        &self,
    ) -> StorageResult<nebari::Tree<nebari::tree::Versioned, nebari::io::fs::StdFile>> {
        Ok(self
            .roots
            .tree(nebari::tree::Versioned::tree("components"))?)
    }

    // ==================== Entity Operations ====================

    /// Spawn a new entity with a component.
    ///
    /// The entity is created in-memory immediately, and will be persisted
    /// when `commit_tick()` is called.
    pub fn spawn<T: 'static + Send + Sync + Clone + bytemuck::Pod>(
        &mut self,
        component: T,
    ) -> Entity {
        let entity = self.world.spawn(component.clone());

        // Record the mutation
        let component_id = self.world.component_id::<T>().unwrap();
        self.pending
            .push(Mutation::set(entity, component_id, &component));

        entity
    }

    /// Insert a component on an entity.
    ///
    /// Records the mutation for the next `commit_tick()`.
    pub fn insert<T: 'static + Send + Sync + Clone + bytemuck::Pod>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> bool {
        if !self.world.insert(entity, component.clone()) {
            return false;
        }

        let component_id = self.world.component_id::<T>().unwrap();
        self.pending
            .push(Mutation::set(entity, component_id, &component));

        true
    }

    /// Update a component on an entity.
    ///
    /// Records the mutation for the next `commit_tick()`.
    pub fn update<T: 'static + Send + Sync + Clone + bytemuck::Pod>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> bool {
        if !self.world.update(entity, component.clone()) {
            return false;
        }

        let component_id = self.world.component_id::<T>().unwrap();
        self.pending
            .push(Mutation::set(entity, component_id, &component));

        true
    }

    /// Remove a component from an entity.
    ///
    /// Records the mutation for the next `commit_tick()`.
    pub fn remove<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<T> {
        let result = self.world.remove::<T>(entity)?;

        let component_id = self.world.component_id::<T>().unwrap();
        self.pending.push(Mutation::remove(entity, component_id));

        Some(result)
    }

    /// Get a component from the current tick (in-memory).
    #[must_use]
    pub fn get<T: 'static + Send + Sync + Clone>(&self, entity: Entity) -> Option<T> {
        self.world.get(entity)
    }

    // ==================== Tick Operations ====================

    /// Commit all pending mutations as a new tick.
    ///
    /// This atomically writes all changes to disk and advances the tick counter.
    /// Use this for single-threaded usage.
    pub fn commit_tick(&mut self) -> StorageResult<TickId> {
        let mutations = std::mem::take(&mut self.pending);
        self.commit_mutations(mutations)
    }

    /// Commit mutations from thread-local buffers after RGB parallel phases.
    ///
    /// The `&mut buffers` requirement guarantees no other threads are
    /// currently accessing their buffers.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let buffers = MutationBuffers::new();
    ///
    /// // RGB parallel phases - each thread pushes to its buffer
    /// rayon::scope(|s| {
    ///     for color in Color::ALL {
    ///         s.spawn(|_| {
    ///             buffers.push_set(entity, comp_id, &value);
    ///         });
    ///     }
    /// });
    ///
    /// // After barrier - commit all mutations
    /// world.commit_tick_from_buffers(&mut buffers)?;
    /// ```
    pub fn commit_tick_from_buffers(
        &mut self,
        buffers: &mut MutationBuffers,
    ) -> StorageResult<TickId> {
        let mutations = buffers.collect_all();
        self.commit_mutations(mutations)
    }

    /// Internal: commit a batch of mutations.
    fn commit_mutations(&mut self, mutations: Vec<Mutation>) -> StorageResult<TickId> {
        self.current_tick += 1;
        let tree = self.tree()?;

        // Apply all mutations
        for mutation in mutations {
            match mutation {
                Mutation::Set { key, data } => {
                    let key_bytes: Vec<u8> = key.as_bytes().to_vec();
                    tree.set(key_bytes, data)?;
                }
                Mutation::Remove { key } => {
                    let key_bytes: Vec<u8> = key.as_bytes().to_vec();
                    tree.remove(&key_bytes)?;
                }
            }
        }

        // Store the current tick
        tree.set(
            b"__tick__".to_vec(),
            self.current_tick.to_le_bytes().to_vec(),
        )?;

        Ok(self.current_tick)
    }

    // ==================== Time Travel ====================

    /// Get a component's value at a specific tick.
    ///
    /// This reads from the versioned storage by scanning the sequence history.
    ///
    /// Note: For the current implementation, this only works for the current tick.
    /// Full time-travel requires scanning sequences or a tick index.
    pub fn get_at_tick<T: 'static + Send + Sync + Clone + bytemuck::Pod>(
        &self,
        entity: Entity,
        tick: TickId,
    ) -> StorageResult<Option<T>> {
        // For the current tick, just use the in-memory world
        if tick == self.current_tick {
            return Ok(self.world.get(entity));
        }

        let tree = self.tree()?;

        let component_id = match self.world.component_id::<T>() {
            Some(id) => id,
            None => return Ok(None),
        };

        let key = ComponentKey::new(entity, component_id);
        let key_bytes: Vec<u8> = key.as_bytes().to_vec();

        // TODO: Implement proper historical lookup using scan_sequences
        // For now, just return the current value from storage
        if let Some(data) = tree.get(&key_bytes)? {
            if let Ok(component) = bytemuck::try_from_bytes::<T>(data.as_ref()) {
                return Ok(Some(*component));
            }
        }

        Ok(None)
    }

    /// Get a component from persistent storage (not in-memory).
    ///
    /// This is useful for verifying persistence or after a restart.
    pub fn get_from_storage<T: 'static + Send + Sync + Clone + bytemuck::Pod>(
        &self,
        entity: Entity,
    ) -> StorageResult<Option<T>> {
        let component_id = match self.world.component_id::<T>() {
            Some(id) => id,
            None => return Ok(None),
        };

        let tree = self.tree()?;
        let key = ComponentKey::new(entity, component_id);
        let key_bytes: Vec<u8> = key.as_bytes().to_vec();

        if let Some(data) = tree.get(&key_bytes)? {
            if let Ok(component) = bytemuck::try_from_bytes::<T>(data.as_ref()) {
                return Ok(Some(*component));
            }
        }

        Ok(None)
    }

    /// Revert the world to a specific tick.
    ///
    /// This restores the in-memory world state to match the persisted state
    /// at the given tick.
    ///
    /// Note: Ticks after the target are still in storage and accessible via
    /// `get_at_tick`, but new commits will increment from the reverted tick.
    pub fn revert_to_tick(&mut self, tick: TickId) -> StorageResult<()> {
        if tick > self.current_tick {
            return Err(crate::error::StorageError::InvalidTick(tick));
        }

        // TODO: Implement full world restoration
        // This requires:
        // 1. Clearing the in-memory world
        // 2. Scanning all keys at the target tick
        // 3. Reconstructing all entities and components
        //
        // For now, just update the tick counter
        self.current_tick = tick;
        self.pending.clear();

        todo!("Full world state restoration not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
    #[repr(C)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
    #[repr(C)]
    struct Health {
        current: u32,
        max: u32,
    }

    #[test]
    fn test_basic_operations() {
        let dir = tempfile::tempdir().unwrap();
        let mut world = VersionedWorld::open(dir.path()).unwrap();

        // Spawn entity
        let player = world.spawn(Position {
            x: 0.0,
            y: 64.0,
            z: 0.0,
        });
        world.insert(
            player,
            Health {
                current: 20,
                max: 20,
            },
        );

        // Read back from in-memory
        let pos = world.get::<Position>(player).unwrap();
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 64.0);

        // Commit tick
        let tick1 = world.commit_tick().unwrap();
        assert_eq!(tick1, 1);

        // Read from storage
        let pos_storage = world.get_from_storage::<Position>(player).unwrap().unwrap();
        assert_eq!(pos_storage.x, 0.0);
    }

    #[test]
    fn test_persistence() {
        let dir = tempfile::tempdir().unwrap();

        // Create and populate world
        {
            let mut world = VersionedWorld::open(dir.path()).unwrap();
            let _player = world.spawn(Position {
                x: 100.0,
                y: 64.0,
                z: 200.0,
            });
            world.commit_tick().unwrap();
        }

        // Reopen and verify tick persisted
        {
            let world = VersionedWorld::open(dir.path()).unwrap();
            assert_eq!(world.current_tick(), 1);
        }
    }
}
