#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::if_not_else)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::manual_is_multiple_of)]

//! Minecraft server using RGB ECS
//!
//! This server uses the custom RGB ECS instead of Flecs.
//! All game state is stored on Entity::WORLD or individual entities.

mod audio;
mod components;
mod network;
mod protocol;
mod registry;
mod systems;
mod world_gen;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use rgb_ecs::prelude::*;
use tracing::info;

use rgb_event::EventPlugin;

use crate::components::*;
use crate::network::NetworkChannels;

fn main() -> eyre::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_server_runner=info".parse()?),
        )
        .init();

    info!("Starting Minecraft server with RGB ECS");

    // Create ECS world with plugins
    let mut world = World::new();

    // Add event system plugin
    world.add_plugin(EventPlugin);

    // Register transient components (not persisted to storage)
    // These are runtime-only: network buffers, channels, caches
    world.register_transient::<PacketBuffer>();
    world.register_transient::<PendingPackets>();
    world.register_transient::<NetworkIngress>();
    world.register_transient::<NetworkEgress>();
    world.register_transient::<DisconnectIngress>();
    world.register_transient::<Connection>();
    world.register_transient::<ProtocolState>();

    // Initialize global state on Entity::WORLD (auto-registers component types)
    world.insert(Entity::WORLD, ServerConfig::default());
    world.insert(Entity::WORLD, WorldTime::default());
    world.insert(Entity::WORLD, TpsTracker::default());
    world.insert(Entity::WORLD, EntityIdCounter::default());
    world.insert(Entity::WORLD, PendingPackets::default());

    // Create network channels and start network thread
    let channels = NetworkChannels::new();
    world.insert(
        Entity::WORLD,
        NetworkIngress {
            rx: channels.ingress_rx.clone(),
        },
    );
    world.insert(
        Entity::WORLD,
        NetworkEgress {
            tx: channels.egress_tx.clone(),
        },
    );
    world.insert(
        Entity::WORLD,
        DisconnectIngress {
            rx: channels.disconnect_rx.clone(),
        },
    );

    // Start network thread
    network::start_network_thread(
        channels.ingress_tx,
        channels.egress_rx,
        channels.disconnect_tx,
    );

    // Generate spawn chunks
    world_gen::generate_spawn_chunks(&mut world, 8);

    info!("Server initialized");

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // Run game loop at 20 TPS
    let target_fps: f32 = std::env::var("TARGET_FPS")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(20.0);
    let target_delta = Duration::from_secs_f32(1.0 / target_fps);
    let mut last_tick = Instant::now();

    while running.load(Ordering::SeqCst) {
        let start = Instant::now();

        // Calculate delta time
        let delta_time = last_tick.elapsed().as_secs_f32();
        last_tick = Instant::now();

        // Run all systems
        systems::tick(&mut world, delta_time);

        // Sleep to maintain target FPS
        let elapsed = start.elapsed();
        if elapsed < target_delta {
            thread::sleep(target_delta - elapsed);
        }
    }

    info!("Shutting down...");
    Ok(())
}
