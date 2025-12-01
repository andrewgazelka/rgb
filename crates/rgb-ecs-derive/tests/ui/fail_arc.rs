//! Test that Arc<T> is forbidden in components.

use rgb_ecs_derive::Component;
use std::sync::Arc;

#[derive(Component, Clone)]
struct SharedData {
    data: Arc<[u8]>,
}

fn main() {}
