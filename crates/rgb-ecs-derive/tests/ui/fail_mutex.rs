//! Test that Mutex<T> is forbidden in components.

use rgb_ecs_derive::Component;
use std::sync::Mutex;

#[derive(Component)]
struct SharedState {
    counter: Mutex<u32>,
}

fn main() {}
