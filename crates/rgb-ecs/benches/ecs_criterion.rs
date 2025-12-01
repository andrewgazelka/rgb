//! ECS benchmarks using criterion for historical comparison.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rgb_ecs::{Entity, World};

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

fn spawn_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn");

    for count in [1, 100, 1000, 10000] {
        group.throughput(Throughput::Elements(count));

        group.bench_with_input(BenchmarkId::new("empty", count), &count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                for _ in 0..count {
                    black_box(world.spawn_empty());
                }
            });
        });

        group.bench_with_input(
            BenchmarkId::new("with_position", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let mut world = World::new();
                    for i in 0..count {
                        black_box(world.spawn(Position {
                            x: i as f32,
                            y: 0.0,
                            z: 0.0,
                        }));
                    }
                });
            },
        );
    }

    group.finish();
}

fn component_access_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_access");

    for count in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(count));

        group.bench_with_input(BenchmarkId::new("get", count), &count, |b, &count| {
            let mut world = World::new();
            let entities: Vec<Entity> = (0..count)
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
            });
        });

        group.bench_with_input(BenchmarkId::new("get_mut", count), &count, |b, &count| {
            let mut world = World::new();
            let entities: Vec<Entity> = (0..count)
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
            });
        });
    }

    group.finish();
}

fn archetype_change_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("archetype_change");

    for count in [100, 1000] {
        group.throughput(Throughput::Elements(count));

        group.bench_with_input(
            BenchmarkId::new("insert_component", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let mut world = World::new();
                    let entities: Vec<Entity> = (0..count)
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
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("remove_component", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut world = World::new();
                        let entities: Vec<Entity> = (0..count)
                            .map(|i| {
                                let e = world.spawn(Position {
                                    x: i as f32,
                                    y: 0.0,
                                    z: 0.0,
                                });
                                world.insert(
                                    e,
                                    Velocity {
                                        x: 1.0,
                                        y: 0.0,
                                        z: 0.0,
                                    },
                                );
                                e
                            })
                            .collect();
                        (world, entities)
                    },
                    |(mut world, entities)| {
                        for entity in entities {
                            world.remove::<Velocity>(entity);
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    spawn_benchmarks,
    component_access_benchmarks,
    archetype_change_benchmarks,
);

criterion_main!(benches);
