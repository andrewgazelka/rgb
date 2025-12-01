//! Test that references are forbidden in components.

use rgb_ecs_derive::Component;

#[derive(Component)]
struct WithRef<'a> {
    data: &'a [u8],
}

fn main() {}
