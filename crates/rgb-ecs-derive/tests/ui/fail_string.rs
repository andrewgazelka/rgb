//! Test that String is forbidden in components.

use rgb_ecs_derive::Component;

#[derive(Component, Clone)]
struct PlayerName {
    name: String,
}

fn main() {}
