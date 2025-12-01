//! Test that Box<T> is forbidden in components.

use rgb_ecs_derive::Component;

#[derive(Component, Clone)]
struct HeapData {
    data: Box<[u8; 1024]>,
}

fn main() {}
