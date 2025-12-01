//! Test that fixed-size arrays are allowed in components.

use rgb_ecs_derive::Component;

#[derive(Component, Clone)]
struct FixedBuffer {
    data: [u8; 32],
}

#[derive(Component, Clone)]
struct Matrix3x3 {
    values: [[f32; 3]; 3],
}

#[derive(Component, Clone, Copy)]
struct ChunkKey {
    bytes: [u8; 13],
}

#[derive(Component, Clone, Copy)]
struct Uuid {
    bytes: [u8; 16],
}

fn main() {}
