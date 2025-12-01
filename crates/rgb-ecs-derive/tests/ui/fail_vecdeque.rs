//! Test that VecDeque<T> is forbidden in components.

use rgb_ecs_derive::Component;
use std::collections::VecDeque;

#[derive(Component, Clone)]
struct PacketBuffer {
    incoming: VecDeque<u8>,
}

fn main() {}
