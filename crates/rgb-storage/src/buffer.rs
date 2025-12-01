//! Thread-local mutation buffers for lock-free parallel writes.
//!
//! During RGB parallel phases, each thread accumulates mutations in its own
//! buffer without any synchronization. After all phases complete, the main
//! thread collects all buffers and commits them to Nebari.
//!
//! Uses the `thread_local` crate for automatic per-thread storage with
//! iteration support.

use rgb_ecs::{ComponentId, Entity};

use crate::ComponentKey;

/// A mutation to be persisted.
#[derive(Debug, Clone)]
pub enum Mutation {
    /// Insert or update a component.
    Set { key: ComponentKey, data: Vec<u8> },
    /// Remove a component.
    Remove { key: ComponentKey },
}

impl Mutation {
    /// Create a Set mutation.
    #[inline]
    pub fn set<T: bytemuck::Pod>(entity: Entity, component_id: ComponentId, value: &T) -> Self {
        Self::Set {
            key: ComponentKey::new(entity, component_id),
            data: bytemuck::bytes_of(value).to_vec(),
        }
    }

    /// Create a Remove mutation.
    #[inline]
    pub const fn remove(entity: Entity, component_id: ComponentId) -> Self {
        Self::Remove {
            key: ComponentKey::new(entity, component_id),
        }
    }

    /// Get the key for this mutation.
    #[inline]
    pub const fn key(&self) -> &ComponentKey {
        match self {
            Self::Set { key, .. } | Self::Remove { key } => key,
        }
    }
}

/// Thread-local mutation buffers.
///
/// Each thread automatically gets its own `Vec<Mutation>` when it first
/// accesses the buffer. After parallel work completes, call `collect_all()`
/// to gather mutations from all threads.
pub struct MutationBuffers {
    inner: thread_local::ThreadLocal<core::cell::RefCell<Vec<Mutation>>>,
}

impl MutationBuffers {
    /// Create a new empty buffer collection.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: thread_local::ThreadLocal::new(),
        }
    }

    /// Get the current thread's mutation buffer.
    ///
    /// Creates an empty buffer if this thread hasn't accessed it yet.
    #[inline]
    pub fn current(&self) -> &core::cell::RefCell<Vec<Mutation>> {
        self.inner.get_or_default()
    }

    /// Push a mutation to the current thread's buffer.
    #[inline]
    pub fn push(&self, mutation: Mutation) {
        self.current().borrow_mut().push(mutation);
    }

    /// Push a Set mutation to the current thread's buffer.
    #[inline]
    pub fn push_set<T: bytemuck::Pod>(&self, entity: Entity, component_id: ComponentId, value: &T) {
        self.push(Mutation::set(entity, component_id, value));
    }

    /// Push a Remove mutation to the current thread's buffer.
    #[inline]
    pub fn push_remove(&self, entity: Entity, component_id: ComponentId) {
        self.push(Mutation::remove(entity, component_id));
    }

    /// Collect all mutations from all threads, clearing the buffers.
    ///
    /// This requires `&mut self` which guarantees no other threads are
    /// currently accessing their buffers.
    pub fn collect_all(&mut self) -> Vec<Mutation> {
        self.inner
            .iter_mut()
            .flat_map(|cell| cell.get_mut().drain(..))
            .collect()
    }

    /// Get total pending mutation count across all threads.
    ///
    /// Requires `&mut self` to ensure no concurrent access.
    pub fn total_pending(&mut self) -> usize {
        self.inner.iter_mut().map(|cell| cell.get_mut().len()).sum()
    }

    /// Clear all buffers.
    ///
    /// Requires `&mut self` to ensure no concurrent access.
    pub fn clear(&mut self) {
        for cell in self.inner.iter_mut() {
            cell.get_mut().clear();
        }
    }
}

impl Default for MutationBuffers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use rgb_ecs::Generation;

    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
    #[repr(C)]
    struct TestComponent {
        value: u32,
    }

    #[test]
    fn test_single_thread() {
        let mut buffers = MutationBuffers::new();
        let entity = Entity::new(1, Generation::new());
        let comp_id = ComponentId::from_raw(42);

        buffers.push_set(entity, comp_id, &TestComponent { value: 100 });
        buffers.push_remove(entity, comp_id);

        assert_eq!(buffers.total_pending(), 2);

        let mutations = buffers.collect_all();
        assert_eq!(mutations.len(), 2);
        assert_eq!(buffers.total_pending(), 0);
    }

    #[test]
    fn test_mutation_key() {
        let entity = Entity::new(42, Generation::new());
        let comp_id = ComponentId::from_raw(123);

        let set = Mutation::set(entity, comp_id, &TestComponent { value: 999 });
        assert_eq!(set.key().entity(), entity);
        assert_eq!(set.key().component_id(), comp_id);

        let remove = Mutation::remove(entity, comp_id);
        assert_eq!(remove.key().entity(), entity);
        assert_eq!(remove.key().component_id(), comp_id);
    }

    #[test]
    fn test_multi_thread() {
        use std::sync::Arc;

        let buffers = Arc::new(MutationBuffers::new());
        let entity = Entity::new(1, Generation::new());
        let comp_id = ComponentId::from_raw(42);

        // Spawn threads that each push mutations
        let handles: Vec<_> = (0..4)
            .map(|i| {
                let buffers = Arc::clone(&buffers);
                std::thread::spawn(move || {
                    buffers.push_set(entity, comp_id, &TestComponent { value: i });
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Unwrap Arc to get mutable access
        let mut buffers = Arc::try_unwrap(buffers).ok().unwrap();
        let mutations = buffers.collect_all();
        assert_eq!(mutations.len(), 4);
    }
}
