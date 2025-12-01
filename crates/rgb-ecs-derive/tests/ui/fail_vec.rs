//! Test that Vec<T> is forbidden in components.

use rgb_ecs_derive::Component;

#[derive(Component, Clone)]
struct Inventory {
    items: Vec<u32>,
}

fn main() {}
