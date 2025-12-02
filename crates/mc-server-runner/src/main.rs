#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::if_not_else)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::manual_is_multiple_of)]

//! Minecraft server using Flecs ECS
//!
//! This server uses Flecs ECS with a pipeline-based system architecture.

// mod audio;
mod components;
#[cfg(feature = "dashboard")]
mod dashboard;
mod network;
mod protocol;
mod registry;
mod systems;
mod world_gen;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use flecs_ecs::prelude::*;
use tracing::info;

use crate::components::*;
use crate::network::NetworkChannels;
use crate::systems::ServerModule;

fn main() -> eyre::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_server_runner=info".parse()?),
        )
        .init();

    info!("Starting Minecraft server with Flecs ECS");

    // Create ECS world
    let world = World::new();

    // Import the server module (registers all components and systems)
    world.import::<ServerModule>();

    // Create network channels and start network thread
    let channels = NetworkChannels::new();

    // Set singletons
    world.set(ServerConfig::default());
    world.set(WorldTime::default());
    world.set(TpsTracker::default());
    world.set(DeltaTime::default());
    world.set(EntityIdCounter::default());
    world.set(PendingPackets::default());
    world.set(ConnectionIndex::default());
    world.set(NetworkIngress {
        rx: channels.ingress_rx.clone(),
    });
    world.set(NetworkEgress {
        tx: channels.egress_tx.clone(),
    });
    world.set(DisconnectIngress {
        rx: channels.disconnect_rx.clone(),
    });

    // Start network thread
    network::start_network_thread(
        channels.ingress_tx,
        channels.egress_rx,
        channels.disconnect_tx,
    );

    // Generate spawn chunks
    world_gen::generate_spawn_chunks(&world, 8);

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

        // Update delta time singleton
        world.set(DeltaTime(delta_time));

        // Run all systems via Flecs pipeline
        world.progress();

        // Sleep to maintain target FPS
        let elapsed = start.elapsed();
        if elapsed < target_delta {
            thread::sleep(target_delta - elapsed);
        }
    }

    info!("Shutting down...");
    Ok(())
}
