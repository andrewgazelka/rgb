//! Parallel tick execution using RGB coloring

use flecs_ecs::prelude::*;

use crate::region::{Chunk, Region, RegionColor};
use crate::scoped::ScopedWorld;

/// Tick execution phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickPhase {
    /// Sequential phase before parallel execution (network ingress, etc.)
    PreGlobal,
    /// Parallel phase for regions of a specific color
    Color(RegionColor),
    /// Sequential phase after parallel execution (network egress, etc.)
    PostGlobal,
}

/// Scheduler for RGB parallel tick execution
pub struct RgbScheduler {
    /// Number of chunks per region (default: 16)
    chunks_per_region: i32,
}

impl Default for RgbScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl RgbScheduler {
    /// Create a new RGB scheduler
    #[must_use]
    pub const fn new() -> Self {
        Self {
            chunks_per_region: 16,
        }
    }

    /// Create with custom chunks per region
    #[must_use]
    pub const fn with_chunks_per_region(chunks_per_region: i32) -> Self {
        Self { chunks_per_region }
    }

    /// Run a complete tick with the given system functions (sequential version)
    ///
    /// This is the sequential version for simpler testing. Use `tick_parallel` for actual
    /// parallel execution with rayon.
    ///
    /// # Arguments
    /// * `world` - The Flecs world
    /// * `pre_global` - Sequential function to run before parallel phases
    /// * `chunk_system` - Function to run for each chunk (receives ScopedWorld)
    /// * `post_global` - Sequential function to run after parallel phases
    pub fn tick<F, G, H>(&self, world: &World, pre_global: F, chunk_system: G, post_global: H)
    where
        F: FnOnce(&World),
        G: Fn(&ScopedWorld<'_>, EntityView<'_>),
        H: FnOnce(&World),
    {
        // 1. Pre-global phase (sequential)
        pre_global(world);

        // 2-4. RGB phases (sequential for now)
        for color in RegionColor::all() {
            Self::run_color_phase_sequential(world, color, &chunk_system);
        }

        // 5. Post-global phase (sequential)
        post_global(world);
    }

    /// Run a single color phase sequentially
    fn run_color_phase_sequential<G>(world: &World, color: RegionColor, chunk_system: &G)
    where
        G: Fn(&ScopedWorld<'_>, EntityView<'_>),
    {
        // Query all regions of this color
        let mut region_ids: Vec<Entity> = Vec::new();

        world
            .query::<&Region>()
            .build()
            .each_entity(|entity, region| {
                if region.color() == color {
                    region_ids.push(entity.id());
                }
            });

        if region_ids.is_empty() {
            return;
        }

        // Process each region
        for region_id in region_ids {
            let region = world.entity_from_id(region_id);
            Self::process_region_chunks_sequential(world, region, chunk_system);
        }
    }

    /// Process all chunks in a region sequentially
    fn process_region_chunks_sequential<G>(world: &World, region: EntityView<'_>, chunk_system: &G)
    where
        G: Fn(&ScopedWorld<'_>, EntityView<'_>),
    {
        // Collect chunk IDs first to avoid lifetime issues
        let mut chunk_ids: Vec<Entity> = Vec::new();

        region.each_child(|chunk_entity| {
            if chunk_entity.try_get::<&Chunk>(|_| ()).is_some() {
                chunk_ids.push(chunk_entity.id());
            }
        });

        // Process each chunk
        for chunk_id in chunk_ids {
            let chunk_entity = world.entity_from_id(chunk_id);
            if let Some(chunk) = chunk_entity.try_get::<&Chunk>(|c| *c) {
                let chunk_pos = (chunk.x, chunk.z);
                let scoped = ScopedWorld::new(world.world(), chunk_pos);
                chunk_system(&scoped, chunk_entity);
            }
        }
    }

    /// Create a region entity with the correct color
    pub fn create_region<'a>(&self, world: &'a World, rx: i32, rz: i32) -> EntityView<'a> {
        let region = Region::new(rx, rz);
        let color = region.color();

        world.entity().set(region).set(color)
    }

    /// Create a chunk entity as child of a region
    pub fn create_chunk<'a>(&self, world: &'a World, x: i32, z: i32) -> EntityView<'a> {
        let chunk = Chunk::new(x, z);
        let (rx, rz) = chunk.region_coords(self.chunks_per_region);

        // Find or create the parent region
        let region_id = self.find_or_create_region_id(world, rx, rz);

        // Create chunk as child of region
        world.entity().set(chunk).child_of(region_id)
    }

    /// Find an existing region or create a new one, returning its ID
    fn find_or_create_region_id(&self, world: &World, rx: i32, rz: i32) -> Entity {
        // Try to find existing region
        let mut found_id: Option<Entity> = None;

        world
            .query::<&Region>()
            .build()
            .each_entity(|entity, region| {
                if region.rx == rx && region.rz == rz {
                    found_id = Some(entity.id());
                }
            });

        if let Some(id) = found_id {
            id
        } else {
            self.create_region(world, rx, rz).id()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_create_region_with_color() {
        let world = World::new();
        let scheduler = RgbScheduler::new();

        let region = scheduler.create_region(&world, 0, 0);
        assert!(region.try_get::<&Region>(|_| ()).is_some());
        assert!(region.try_get::<&RegionColor>(|_| ()).is_some());
    }

    #[test]
    fn test_create_chunk_hierarchy() {
        let world = World::new();
        let scheduler = RgbScheduler::new();

        // Create a chunk at (0, 0)
        let chunk = scheduler.create_chunk(&world, 0, 0);
        assert!(chunk.try_get::<&Chunk>(|_| ()).is_some());

        // Create another chunk in the same region
        let chunk2 = scheduler.create_chunk(&world, 1, 1);
        assert!(chunk2.try_get::<&Chunk>(|_| ()).is_some());

        // Both should be children of the same region
        // (they're both in region (0, 0) since chunks_per_region = 16)
    }

    #[test]
    fn test_tick_execution() {
        let world = World::new();
        let scheduler = RgbScheduler::new();

        // Create some regions and chunks
        let _chunk1 = scheduler.create_chunk(&world, 0, 0);
        let _chunk2 = scheduler.create_chunk(&world, 16, 0); // Different region
        let _chunk3 = scheduler.create_chunk(&world, 32, 0); // Another region

        // Create some entities with positions
        for i in 0..10 {
            world
                .entity()
                .set(Position::new(f64::from(i) * 10.0, 64.0, 0.0));
        }

        // Track how many times chunk_system is called
        let call_count = AtomicU32::new(0);

        scheduler.tick(
            &world,
            |_world| {
                // Pre-global: nothing for this test
            },
            |_scoped, _chunk| {
                call_count.fetch_add(1, Ordering::Relaxed);
            },
            |_world| {
                // Post-global: nothing for this test
            },
        );

        // Should have processed 3 chunks
        assert_eq!(call_count.load(Ordering::Relaxed), 3);
    }
}
