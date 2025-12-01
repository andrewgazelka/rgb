//! Test that tuples are allowed in components.

use rgb_ecs_derive::Component;

#[derive(Component, Clone, Copy)]
struct Point2D(f32, f32);

#[derive(Component, Clone, Copy)]
struct WithTuple {
    coords: (i32, i32),
}

#[derive(Component, Clone, Copy)]
struct Unit;

fn main() {}
