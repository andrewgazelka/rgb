//! Test that HashMap<K, V> is forbidden in components.

use rgb_ecs_derive::Component;
use std::collections::HashMap;

#[derive(Component, Clone)]
struct PlayerData {
    attributes: HashMap<u32, f32>,
}

fn main() {}
