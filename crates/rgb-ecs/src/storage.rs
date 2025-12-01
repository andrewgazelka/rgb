//! Component storage - type-erased column storage for archetype tables.
//!
//! Each column stores components of a single type in a contiguous array,
//! enabling cache-friendly iteration.

use std::{alloc::Layout, ptr::NonNull};

use crate::component::ComponentInfo;

/// A column of components of a single type.
///
/// Stores components in a contiguous, type-erased array.
/// Manages its own memory allocation and deallocation.
pub struct Column {
    /// Pointer to the data array.
    data: NonNull<u8>,
    /// Number of components stored.
    len: usize,
    /// Allocated capacity (in number of components).
    capacity: usize,
    /// Component type information.
    info: ComponentInfo,
}

// SAFETY: Column manages its own memory and ComponentInfo ensures
// the stored type is Send + Sync
unsafe impl Send for Column {}
unsafe impl Sync for Column {}

impl Column {
    /// Create a new empty column for the given component type.
    #[must_use]
    pub fn new(info: ComponentInfo) -> Self {
        Self {
            data: NonNull::dangling(),
            len: 0,
            capacity: 0,
            info,
        }
    }

    /// Create a column with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(info: ComponentInfo, capacity: usize) -> Self {
        if capacity == 0 || info.size() == 0 {
            return Self::new(info);
        }

        let layout = Self::array_layout(&info, capacity);

        // SAFETY: Layout is valid and non-zero
        let data = unsafe {
            let ptr = std::alloc::alloc(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            NonNull::new_unchecked(ptr)
        };

        Self {
            data,
            len: 0,
            capacity,
            info,
        }
    }

    /// Get the number of components stored.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if the column is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the component info.
    #[must_use]
    pub const fn info(&self) -> &ComponentInfo {
        &self.info
    }

    /// Push a component onto the column.
    ///
    /// # Safety
    ///
    /// - `value` must point to a valid instance of the column's component type.
    /// - The memory at `value` will be copied; caller retains ownership of `value`.
    pub unsafe fn push_raw(&mut self, value: *const u8) {
        self.reserve(1);

        // SAFETY: We just reserved space, so self.len is a valid index
        let dst = unsafe { self.get_unchecked_raw(self.len) };

        // SAFETY: dst is valid, value is valid, and they don't overlap
        unsafe {
            std::ptr::copy_nonoverlapping(value, dst, self.info.size());
        }

        self.len += 1;
    }

    /// Push a typed component onto the column.
    ///
    /// # Panics
    ///
    /// Panics if `T` doesn't match the column's component type (in debug builds).
    pub fn push<T: 'static>(&mut self, value: T) {
        debug_assert!(self.info.is::<T>(), "Type mismatch in Column::push");

        // SAFETY: We verified the type matches
        unsafe {
            self.push_raw(std::ptr::from_ref(&value).cast());
        }

        // Don't drop the value since we copied it
        std::mem::forget(value);
    }

    /// Remove and return the component at the given index.
    /// Swaps with the last element for O(1) removal.
    ///
    /// # Safety
    ///
    /// - `index` must be less than `len`.
    /// - `out` must point to valid memory for the component type.
    ///
    /// Returns the index of the entity that was swapped into `index`,
    /// or `None` if `index` was the last element.
    pub unsafe fn swap_remove_raw(&mut self, index: usize, out: *mut u8) -> Option<usize> {
        debug_assert!(index < self.len, "Index out of bounds in swap_remove");

        // SAFETY: Caller ensures index is valid
        let src = unsafe { self.get_unchecked_raw(index) };

        // Copy the removed value to output
        // SAFETY: Caller ensures out is valid
        unsafe {
            std::ptr::copy_nonoverlapping(src, out, self.info.size());
        }

        self.len -= 1;

        if index < self.len {
            // Swap with last element
            // SAFETY: self.len is now < old len, so it's a valid index
            let last = unsafe { self.get_unchecked_raw(self.len) };

            // SAFETY: src and last don't overlap (different indices)
            unsafe {
                std::ptr::copy_nonoverlapping(last, src, self.info.size());
            }

            Some(self.len)
        } else {
            None
        }
    }

    /// Remove and drop the component at the given index.
    /// Swaps with the last element for O(1) removal.
    ///
    /// Returns the index of the entity that was swapped into `index`,
    /// or `None` if `index` was the last element.
    ///
    /// # Safety
    ///
    /// `index` must be less than `len`.
    pub unsafe fn swap_remove_drop(&mut self, index: usize) -> Option<usize> {
        debug_assert!(index < self.len, "Index out of bounds in swap_remove_drop");

        // SAFETY: Caller ensures index is valid
        let ptr = unsafe { self.get_unchecked_raw(index) };

        // Drop the component
        // SAFETY: ptr points to a valid initialized component
        unsafe {
            self.info.drop_in_place(ptr);
        }

        self.len -= 1;

        if index < self.len {
            // Swap with last element
            // SAFETY: self.len is now < old len, so it's a valid index
            let last = unsafe { self.get_unchecked_raw(self.len) };

            // SAFETY: ptr and last don't overlap
            unsafe {
                std::ptr::copy_nonoverlapping(last, ptr, self.info.size());
            }

            Some(self.len)
        } else {
            None
        }
    }

    /// Get a raw pointer to the component at the given index.
    ///
    /// # Safety
    ///
    /// `index` must be less than `len`.
    #[must_use]
    pub unsafe fn get_unchecked_raw(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len || (index == self.len && self.len < self.capacity));
        // SAFETY: Caller ensures index is valid
        unsafe { self.data.as_ptr().add(index * self.info.size()) }
    }

    /// Get a reference to the component at the given index.
    ///
    /// # Safety
    ///
    /// - `index` must be less than `len`.
    /// - `T` must match the column's component type.
    #[must_use]
    pub unsafe fn get_unchecked<T: 'static>(&self, index: usize) -> &T {
        debug_assert!(self.info.is::<T>(), "Type mismatch in Column::get");
        // SAFETY: Caller ensures index is valid and type matches
        unsafe { &*self.get_unchecked_raw(index).cast::<T>() }
    }

    /// Get a mutable reference to the component at the given index.
    ///
    /// # Safety
    ///
    /// - `index` must be less than `len`.
    /// - `T` must match the column's component type.
    /// - No other references to this component may exist.
    #[must_use]
    pub unsafe fn get_unchecked_mut<T: 'static>(&mut self, index: usize) -> &mut T {
        debug_assert!(self.info.is::<T>(), "Type mismatch in Column::get_mut");
        // SAFETY: Caller ensures index is valid, type matches, and no aliasing
        unsafe { &mut *self.get_unchecked_raw(index).cast::<T>() }
    }

    /// Get a pointer to the start of the data array.
    #[must_use]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Get a mutable pointer to the start of the data array.
    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_ptr()
    }

    /// Reserve capacity for at least `additional` more components.
    pub fn reserve(&mut self, additional: usize) {
        let required = self.len.checked_add(additional).expect("Capacity overflow");

        if required <= self.capacity {
            return;
        }

        self.grow(required);
    }

    /// Grow the column to at least `min_capacity`.
    fn grow(&mut self, min_capacity: usize) {
        // Growth strategy: double capacity, but at least 4 elements
        let new_capacity = self
            .capacity
            .checked_mul(2)
            .unwrap_or(min_capacity)
            .max(min_capacity)
            .max(4);

        if self.info.size() == 0 {
            // Zero-sized types don't need allocation
            self.capacity = usize::MAX;
            return;
        }

        let new_layout = Self::array_layout(&self.info, new_capacity);

        // SAFETY: We handle both new allocation and reallocation
        let new_data = unsafe {
            if self.capacity == 0 {
                // New allocation
                let ptr = std::alloc::alloc(new_layout);
                if ptr.is_null() {
                    std::alloc::handle_alloc_error(new_layout);
                }
                ptr
            } else {
                // Reallocation
                let old_layout = Self::array_layout(&self.info, self.capacity);
                let ptr = std::alloc::realloc(self.data.as_ptr(), old_layout, new_layout.size());
                if ptr.is_null() {
                    std::alloc::handle_alloc_error(new_layout);
                }
                ptr
            }
        };

        self.data = NonNull::new(new_data).expect("Allocation returned null");
        self.capacity = new_capacity;
    }

    /// Clear all components, dropping them.
    pub fn clear(&mut self) {
        if self.info.needs_drop() {
            for i in 0..self.len {
                // SAFETY: i is valid index
                let ptr = unsafe { self.get_unchecked_raw(i) };
                // SAFETY: ptr points to valid initialized component
                unsafe { self.info.drop_in_place(ptr) };
            }
        }
        self.len = 0;
    }

    /// Calculate the array layout for `count` components.
    fn array_layout(info: &ComponentInfo, count: usize) -> Layout {
        let size = info.size().checked_mul(count).expect("Layout overflow");
        // SAFETY: align is always a power of 2 from Layout
        unsafe { Layout::from_size_align_unchecked(size, info.align()) }
    }
}

impl Drop for Column {
    fn drop(&mut self) {
        // Drop all components
        self.clear();

        // Deallocate memory
        if self.capacity > 0 && self.info.size() > 0 {
            let layout = Self::array_layout(&self.info, self.capacity);
            // SAFETY: data was allocated with this layout
            unsafe {
                std::alloc::dealloc(self.data.as_ptr(), layout);
            }
        }
    }
}

/// Generic component storage trait for different storage strategies.
pub trait ComponentStorage {
    /// Check if the storage contains the given component type.
    fn contains(&self, id: crate::ComponentId) -> bool;

    /// Get the number of entities in storage.
    fn len(&self) -> usize;

    /// Check if storage is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComponentId;

    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Name(String);

    #[test]
    fn test_column_push_get() {
        let info = ComponentInfo::of::<Position>(ComponentId::from_raw(0));
        let mut col = Column::new(info);

        col.push(Position { x: 1.0, y: 2.0 });
        col.push(Position { x: 3.0, y: 4.0 });

        assert_eq!(col.len(), 2);

        // SAFETY: Valid indices and correct type
        unsafe {
            assert_eq!(
                col.get_unchecked::<Position>(0),
                &Position { x: 1.0, y: 2.0 }
            );
            assert_eq!(
                col.get_unchecked::<Position>(1),
                &Position { x: 3.0, y: 4.0 }
            );
        }
    }

    #[test]
    fn test_column_swap_remove() {
        let info = ComponentInfo::of::<Position>(ComponentId::from_raw(0));
        let mut col = Column::new(info);

        col.push(Position { x: 1.0, y: 2.0 });
        col.push(Position { x: 3.0, y: 4.0 });
        col.push(Position { x: 5.0, y: 6.0 });

        let mut removed = Position { x: 0.0, y: 0.0 };

        // SAFETY: Index 0 is valid
        let swapped = unsafe { col.swap_remove_raw(0, std::ptr::from_mut(&mut removed).cast()) };

        assert_eq!(removed, Position { x: 1.0, y: 2.0 });
        assert_eq!(swapped, Some(2)); // Last element was at index 2
        assert_eq!(col.len(), 2);

        // SAFETY: Valid indices
        unsafe {
            // Element from index 2 is now at index 0
            assert_eq!(
                col.get_unchecked::<Position>(0),
                &Position { x: 5.0, y: 6.0 }
            );
            assert_eq!(
                col.get_unchecked::<Position>(1),
                &Position { x: 3.0, y: 4.0 }
            );
        }
    }

    #[test]
    fn test_column_with_drop() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;

        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        DROP_COUNT.store(0, Ordering::SeqCst);

        {
            let info = ComponentInfo::of::<DropCounter>(ComponentId::from_raw(0));
            let mut col = Column::new(info);

            col.push(DropCounter);
            col.push(DropCounter);
            col.push(DropCounter);

            assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 0);
        }

        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_column_string() {
        let info = ComponentInfo::of::<Name>(ComponentId::from_raw(0));
        let mut col = Column::new(info);

        col.push(Name("Hello".to_string()));
        col.push(Name("World".to_string()));

        // SAFETY: Valid indices and correct type
        unsafe {
            assert_eq!(col.get_unchecked::<Name>(0).0, "Hello");
            assert_eq!(col.get_unchecked::<Name>(1).0, "World");
        }
    }
}
