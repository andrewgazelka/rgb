//! Test that opaque components skip validation.

use rgb_ecs_derive::Component;
use std::collections::VecDeque;
use std::sync::Arc;

// Opaque component - allows forbidden types
#[derive(Component, Clone)]
#[component(opaque)]
struct PacketBuffer {
    incoming: VecDeque<u8>,
    outgoing: VecDeque<u8>,
}

// Opaque with Arc
#[derive(Component, Clone)]
#[component(opaque)]
struct SharedData {
    data: Arc<[u8]>,
}

// Opaque with String
#[derive(Component, Clone)]
#[component(opaque)]
struct RuntimeName {
    name: String,
}

fn main() {}
