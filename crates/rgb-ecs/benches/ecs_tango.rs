//! ECS benchmarks using tango-bench for paired comparison testing.

use std::hint::black_box;

use rgb_ecs::{Entity, World};
use tango_bench::{IntoBenchmarks, benchmark_fn, tango_benchmarks, tango_main};

#[derive(Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy)]
struct Health(u32);

fn spawn_benchmarks() -> impl IntoBenchmarks {
    [
        benchmark_fn("spawn_empty/1", |b| {
            b.iter(|| {
                let mut world = World::new();
                black_box(world.spawn_empty());
            })
        }),
        benchmark_fn("spawn_empty/100", |b| {
            b.iter(|| {
                let mut world = World::new();
                for _ in 0..100 {
                    black_box(world.spawn_empty());
                }
            })
        }),
        benchmark_fn("spawn_empty/1000", |b| {
            b.iter(|| {
                let mut world = World::new();
                for _ in 0..1000 {
                    black_box(world.spawn_empty());
                }
            })
        }),
        benchmark_fn("spawn_with_component/1", |b| {
            b.iter(|| {
                let mut world = World::new();
                black_box(world.spawn(Position {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }));
            })
        }),
        benchmark_fn("spawn_with_component/1000", |b| {
            b.iter(|| {
                let mut world = World::new();
                for i in 0..1000 {
                    black_box(world.spawn(Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    }));
                }
            })
        }),
    ]
}

fn component_benchmarks() -> impl IntoBenchmarks {
    [
        benchmark_fn("insert_component/1000", |b| {
            b.iter(|| {
                let mut world = World::new();
                let entities: Vec<Entity> = (0..1000)
                    .map(|i| {
                        world.spawn(Position {
                            x: i as f32,
                            y: 0.0,
                            z: 0.0,
                        })
                    })
                    .collect();

                for entity in entities {
                    world.insert(
                        entity,
                        Velocity {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    );
                }
            })
        }),
        benchmark_fn("get_component/1000", |b| {
            let mut world = World::new();
            let entities: Vec<Entity> = (0..1000)
                .map(|i| {
                    world.spawn(Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    })
                })
                .collect();

            b.iter(|| {
                for &entity in &entities {
                    black_box(world.get::<Position>(entity));
                }
            })
        }),
        benchmark_fn("get_mut_component/1000", |b| {
            let mut world = World::new();
            let entities: Vec<Entity> = (0..1000)
                .map(|i| {
                    world.spawn(Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    })
                })
                .collect();

            b.iter(|| {
                for &entity in &entities {
                    if let Some(pos) = world.get_mut::<Position>(entity) {
                        pos.x += 1.0;
                    }
                }
            })
        }),
    ]
}

fn despawn_benchmarks() -> impl IntoBenchmarks {
    [benchmark_fn("despawn/1000", |b| {
        b.iter(|| {
            let mut world = World::new();
            let entities: Vec<Entity> = (0..1000)
                .map(|i| {
                    world.spawn(Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    })
                })
                .collect();

            for entity in entities {
                world.despawn(entity);
            }
        })
    })]
}

tango_benchmarks!(
    spawn_benchmarks(),
    component_benchmarks(),
    despawn_benchmarks()
);
tango_main!();
