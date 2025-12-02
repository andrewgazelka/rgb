//! Integration tests for flecs-rgb

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use flecs_ecs::prelude::*;
use flecs_rgb::prelude::*;

// ============================================================================
// Test Components
// ============================================================================

#[derive(Component, Clone, Debug, PartialEq)]
struct Health {
    current: u32,
    max: u32,
}

#[derive(Component, Clone, Debug)]
struct Velocity {
    x: f64,
    z: f64,
}

#[derive(Component, Clone, Debug)]
struct Player {
    name: [u8; 16],
}

// ============================================================================
// Region and Chunk Tests
// ============================================================================

#[test]
fn test_region_hierarchy_structure() {
    let world = World::new();
    let scheduler = RgbScheduler::new();

    // Create chunks in different regions
    let chunk_0_0 = scheduler.create_chunk(&world, 0, 0);
    let chunk_1_1 = scheduler.create_chunk(&world, 1, 1);
    let chunk_16_0 = scheduler.create_chunk(&world, 16, 0); // Different region
    let chunk_17_1 = scheduler.create_chunk(&world, 17, 1); // Same region as chunk_16_0

    // Verify chunks have Chunk component
    assert!(chunk_0_0.try_get::<&Chunk>(|_| ()).is_some());
    assert!(chunk_16_0.try_get::<&Chunk>(|_| ()).is_some());

    // Count regions
    let mut region_count = 0;
    world.query::<&Region>().build().each(|_| {
        region_count += 1;
    });

    // Should have 2 regions: one for (0,0)-(15,15) and one for (16,0)-(31,15)
    assert_eq!(region_count, 2, "Expected 2 regions");

    // Verify chunk coordinates
    let c0 = chunk_0_0.try_get::<&Chunk>(|c| (c.x, c.z)).unwrap();
    assert_eq!(c0, (0, 0));

    let c16 = chunk_16_0.try_get::<&Chunk>(|c| (c.x, c.z)).unwrap();
    assert_eq!(c16, (16, 0));
}

#[test]
fn test_negative_coordinates() {
    let world = World::new();
    let scheduler = RgbScheduler::new();

    // Create chunks with negative coordinates
    let chunk_neg = scheduler.create_chunk(&world, -5, -10);
    let chunk_neg2 = scheduler.create_chunk(&world, -20, -5);

    // Verify chunk coordinates are preserved
    let c = chunk_neg.try_get::<&Chunk>(|c| (c.x, c.z)).unwrap();
    assert_eq!(c, (-5, -10));

    // Different regions for different 16x16 blocks
    let mut region_count = 0;
    world.query::<&Region>().build().each(|_| {
        region_count += 1;
    });
    assert_eq!(region_count, 2);
}

#[test]
fn test_region_colors_are_assigned() {
    let world = World::new();
    let scheduler = RgbScheduler::new();

    // Create multiple regions
    for rx in -2..3 {
        for rz in -2..3 {
            scheduler.create_region(&world, rx, rz);
        }
    }

    // Count regions by color
    let mut red_count = 0;
    let mut green_count = 0;
    let mut blue_count = 0;

    world
        .query::<&RegionColor>()
        .build()
        .each(|color| match color {
            RegionColor::Red => red_count += 1,
            RegionColor::Green => green_count += 1,
            RegionColor::Blue => blue_count += 1,
        });

    // Should have approximately equal distribution
    let total = red_count + green_count + blue_count;
    assert_eq!(total, 25); // 5x5 grid
    assert!(red_count > 0);
    assert!(green_count > 0);
    assert!(blue_count > 0);
}

// ============================================================================
// ScopedWorld Tests
// ============================================================================

#[test]
fn test_scoped_world_boundary_enforcement() {
    let world = World::new();

    // Create entities at various positions
    let entity_origin = world.entity().set(Position::new(8.0, 64.0, 8.0)); // chunk (0, 0)
    let entity_neighbor = world.entity().set(Position::new(24.0, 64.0, 8.0)); // chunk (1, 0)
    let entity_far = world.entity().set(Position::new(100.0, 64.0, 100.0)); // chunk (6, 6)

    let scoped = ScopedWorld::new((&world).world(), (0, 0));

    // Origin should be accessible
    assert!(scoped.get::<Position>(entity_origin).is_ok());

    // Neighbor should be accessible (Chebyshev distance = 1)
    assert!(scoped.get::<Position>(entity_neighbor).is_ok());

    // Far entity should NOT be accessible
    let result = scoped.get::<Position>(entity_far);
    assert!(matches!(result, Err(ScopeError::OutOfBounds { .. })));
}

#[test]
fn test_scoped_world_diagonal_access() {
    let world = World::new();

    // Entity at diagonal neighbor chunk (1, 1)
    let entity_diag = world.entity().set(Position::new(24.0, 64.0, 24.0));

    let scoped = ScopedWorld::new((&world).world(), (0, 0));

    // Diagonal neighbors (Chebyshev distance = 1) should be accessible
    assert!(scoped.get::<Position>(entity_diag).is_ok());
}

#[test]
fn test_scoped_world_set_component() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position::new(8.0, 64.0, 8.0))
        .set(Health {
            current: 100,
            max: 100,
        });

    let scoped = ScopedWorld::new((&world).world(), (0, 0));

    // Set a new health value
    let result = scoped.set(
        entity,
        Health {
            current: 50,
            max: 100,
        },
    );
    assert!(result.is_ok());

    // Verify the change
    let health = entity.try_get::<&Health>(|h| h.clone()).unwrap();
    assert_eq!(health.current, 50);
}

#[test]
fn test_scoped_world_custom_distance() {
    let world = World::new();

    // Entity at chunk (3, 3)
    let entity = world.entity().set(Position::new(56.0, 64.0, 56.0));

    // Default distance (1) - should be out of bounds
    let scoped_default = ScopedWorld::new((&world).world(), (0, 0));
    assert!(scoped_default.get::<Position>(entity).is_err());

    // Extended distance (3) - should be accessible
    let scoped_extended = ScopedWorld::with_max_distance((&world).world(), (0, 0), 3);
    assert!(scoped_extended.get::<Position>(entity).is_ok());
}

// ============================================================================
// Tick Execution Tests
// ============================================================================

#[test]
fn test_tick_processes_all_chunks() {
    let world = World::new();
    let scheduler = RgbScheduler::new();

    // Create a 3x3 grid of chunks across 3 regions
    for x in [0, 16, 32] {
        for z in [0, 16, 32] {
            scheduler.create_chunk(&world, x, z);
        }
    }

    let chunk_count = AtomicU32::new(0);

    scheduler.tick(
        &world,
        |_| {},
        |_scoped, _chunk| {
            chunk_count.fetch_add(1, Ordering::Relaxed);
        },
        |_| {},
    );

    assert_eq!(chunk_count.load(Ordering::Relaxed), 9);
}

#[test]
fn test_tick_phases_execute_in_order() {
    let world = World::new();
    let scheduler = RgbScheduler::new();

    scheduler.create_chunk(&world, 0, 0);

    let phase_order = Arc::new(std::sync::Mutex::new(Vec::new()));

    let order1 = Arc::clone(&phase_order);
    let order2 = Arc::clone(&phase_order);
    let order3 = Arc::clone(&phase_order);

    scheduler.tick(
        &world,
        move |_| {
            order1.lock().unwrap().push("pre");
        },
        move |_, _| {
            order2.lock().unwrap().push("chunk");
        },
        move |_| {
            order3.lock().unwrap().push("post");
        },
    );

    let order = phase_order.lock().unwrap();
    assert_eq!(order[0], "pre");
    assert_eq!(order[1], "chunk");
    assert_eq!(order[2], "post");
}

#[test]
fn test_tick_with_entities() {
    let world = World::new();
    let scheduler = RgbScheduler::new();

    // Create a chunk
    scheduler.create_chunk(&world, 0, 0);

    // Create entities with positions in that chunk
    for i in 0..5 {
        world
            .entity()
            .set(Position::new(f64::from(i) * 3.0, 64.0, 0.0))
            .set(Velocity { x: 1.0, z: 0.0 });
    }

    let processed = AtomicU32::new(0);

    scheduler.tick(
        &world,
        |_| {},
        |scoped, _chunk| {
            // In a real system, you'd query entities in this chunk
            // For now, just verify we can access the scoped world
            let _ = scoped.center_chunk();
            processed.fetch_add(1, Ordering::Relaxed);
        },
        |_| {},
    );

    assert_eq!(processed.load(Ordering::Relaxed), 1);
}

// ============================================================================
// Event System Tests
// ============================================================================

struct DamageEvent {
    amount: u32,
    source: u64,
}

impl Event for DamageEvent {}

struct HealEvent {
    amount: u32,
}

impl Event for HealEvent {}

static TOTAL_DAMAGE: AtomicU32 = AtomicU32::new(0);
static TOTAL_HEALING: AtomicU32 = AtomicU32::new(0);

fn handle_damage(
    event_ptr: *const core::ffi::c_void,
    _scoped: &ScopedWorld<'_>,
    _target: EntityView<'_>,
) {
    let event = unsafe { &*event_ptr.cast::<DamageEvent>() };
    TOTAL_DAMAGE.fetch_add(event.amount, Ordering::Relaxed);
}

fn handle_heal(
    event_ptr: *const core::ffi::c_void,
    _scoped: &ScopedWorld<'_>,
    _target: EntityView<'_>,
) {
    let event = unsafe { &*event_ptr.cast::<HealEvent>() };
    TOTAL_HEALING.fetch_add(event.amount, Ordering::Relaxed);
}

#[test]
fn test_event_handler_registration() {
    TOTAL_DAMAGE.store(0, Ordering::Relaxed);

    let world = World::new();

    let player = world
        .entity()
        .set(Position::new(0.0, 64.0, 0.0))
        .set(Health {
            current: 100,
            max: 100,
        });

    // Register damage handler
    world.register_handler::<DamageEvent>(player, handle_damage);

    let scoped = ScopedWorld::new((&world).world(), (0, 0));

    // Dispatch damage events
    world.dispatch(
        player,
        &DamageEvent {
            amount: 10,
            source: 0,
        },
        &scoped,
    );
    world.dispatch(
        player,
        &DamageEvent {
            amount: 25,
            source: 0,
        },
        &scoped,
    );
    world.dispatch(
        player,
        &DamageEvent {
            amount: 15,
            source: 0,
        },
        &scoped,
    );

    assert_eq!(TOTAL_DAMAGE.load(Ordering::Relaxed), 50);
}

#[test]
fn test_multiple_event_types_per_entity() {
    TOTAL_DAMAGE.store(0, Ordering::Relaxed);
    TOTAL_HEALING.store(0, Ordering::Relaxed);

    let world = World::new();

    let player = world
        .entity()
        .set(Position::new(0.0, 64.0, 0.0))
        .set(Health {
            current: 50,
            max: 100,
        });

    // Register both handlers
    world.register_handler::<DamageEvent>(player, handle_damage);
    world.register_handler::<HealEvent>(player, handle_heal);

    let scoped = ScopedWorld::new((&world).world(), (0, 0));

    // Dispatch both event types
    world.dispatch(
        player,
        &DamageEvent {
            amount: 20,
            source: 0,
        },
        &scoped,
    );
    world.dispatch(player, &HealEvent { amount: 30 }, &scoped);
    world.dispatch(
        player,
        &DamageEvent {
            amount: 10,
            source: 0,
        },
        &scoped,
    );

    assert_eq!(TOTAL_DAMAGE.load(Ordering::Relaxed), 30);
    assert_eq!(TOTAL_HEALING.load(Ordering::Relaxed), 30);
}

#[test]
fn test_events_only_dispatch_to_registered_targets() {
    TOTAL_DAMAGE.store(0, Ordering::Relaxed);

    let world = World::new();

    let player1 = world.entity().set(Position::new(0.0, 64.0, 0.0));
    let player2 = world.entity().set(Position::new(16.0, 64.0, 0.0));

    // Only register handler for player1
    world.register_handler::<DamageEvent>(player1, handle_damage);

    let scoped = ScopedWorld::new((&world).world(), (0, 0));

    // Dispatch to player1 (should work)
    world.dispatch(
        player1,
        &DamageEvent {
            amount: 100,
            source: 0,
        },
        &scoped,
    );

    // Dispatch to player2 (no handler, should do nothing)
    world.dispatch(
        player2,
        &DamageEvent {
            amount: 999,
            source: 0,
        },
        &scoped,
    );

    // Only player1's damage should be counted
    assert_eq!(TOTAL_DAMAGE.load(Ordering::Relaxed), 100);
}

// ============================================================================
// Full Integration Test
// ============================================================================

#[test]
fn test_full_tick_with_events() {
    let world = World::new();
    let scheduler = RgbScheduler::new();

    // Create world structure
    scheduler.create_chunk(&world, 0, 0);
    scheduler.create_chunk(&world, 16, 0);

    // Create players with positions
    let player1 = world
        .entity()
        .set(Position::new(8.0, 64.0, 8.0))
        .set(Health {
            current: 100,
            max: 100,
        });

    let player2 = world
        .entity()
        .set(Position::new(24.0, 64.0, 8.0))
        .set(Health {
            current: 100,
            max: 100,
        });

    // Track tick execution
    let pre_called = AtomicU32::new(0);
    let chunks_processed = AtomicU32::new(0);
    let post_called = AtomicU32::new(0);

    scheduler.tick(
        &world,
        |_| {
            pre_called.fetch_add(1, Ordering::Relaxed);
        },
        |scoped, _chunk| {
            chunks_processed.fetch_add(1, Ordering::Relaxed);

            // Verify we can access entities in scope
            let result = scoped.get::<Position>(player1);
            // player1 might or might not be in scope depending on chunk
            let _ = result;
        },
        |_| {
            post_called.fetch_add(1, Ordering::Relaxed);
        },
    );

    assert_eq!(pre_called.load(Ordering::Relaxed), 1);
    assert_eq!(chunks_processed.load(Ordering::Relaxed), 2);
    assert_eq!(post_called.load(Ordering::Relaxed), 1);
}
