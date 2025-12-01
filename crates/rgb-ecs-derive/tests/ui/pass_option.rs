//! Test that Option<T> is allowed when T is allowed.

use rgb_ecs_derive::Component;

#[derive(Component, Clone, Copy)]
struct MaybeHealth {
    value: Option<u32>,
}

#[derive(Component, Clone, Copy)]
struct MaybePosition {
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
}

fn main() {}
