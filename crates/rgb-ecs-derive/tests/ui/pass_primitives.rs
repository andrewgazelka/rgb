//! Test that primitive types are allowed in components.

use rgb_ecs_derive::Component;

#[derive(Component, Clone)]
struct AllPrimitives {
    a: i8,
    b: i16,
    c: i32,
    d: i64,
    e: i128,
    f: isize,
    g: u8,
    h: u16,
    i: u32,
    j: u64,
    k: u128,
    l: usize,
    m: f32,
    n: f64,
    o: bool,
    p: char,
}

#[derive(Component, Clone, Copy)]
struct Position {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Component, Clone, Copy)]
struct Health {
    current: u32,
    max: u32,
}

fn main() {}
