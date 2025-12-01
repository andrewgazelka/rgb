//! Test that enums are allowed as components.

use rgb_ecs_derive::Component;

#[derive(Component, Clone, Copy)]
enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Component, Clone, Copy)]
enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Component, Clone, Copy)]
enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

// Enum with data
#[derive(Component, Clone, Copy)]
enum Event {
    Move { dx: f32, dy: f32 },
    Jump,
    Attack { target_id: u64 },
}

fn main() {}
