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
mod persistence;
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

    // Enable Flecs REST API for explorer (https://www.flecs.dev/explorer)
    let rest_port: u16 = std::env::var("FLECS_REST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(27750);

    // Import stats module for performance monitoring in explorer
    world.import::<flecs_ecs::addons::stats::Stats>();

    world.set(flecs::rest::Rest {
        port: rest_port,
        ..Default::default()
    });
    info!(
        "Flecs REST API enabled on port {} - connect via https://www.flecs.dev/explorer",
        rest_port
    );

    // Initialize component serialization (must be before systems)
    systems::history::init_serialization(&world);

    // Initialize all systems
    systems::init_systems(&world);

    // Create network channels and start network thread
    let channels = NetworkChannels::new();

    // Create persistence database
    let persistence_db = {
        let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string());
        let db_path = std::path::Path::new(&data_dir).join("history.nebari");
        match persistence::PersistenceDb::open(&db_path) {
            Ok(db) => {
                info!("Persistence database opened at {:?}", db_path);
                Some(db)
            }
            Err(e) => {
                tracing::warn!("Failed to open persistence database: {e}");
                None
            }
        }
    };

    // Create persistence channels and register observer
    let persistence_channels = persistence::PersistenceChannels::new();
    persistence::register_persistence_observer(&world, persistence_channels.tx.clone());

    // Create dashboard channels and start dashboard server
    #[cfg(feature = "dashboard")]
    let dashboard_channels = {
        let channels = dashboard::DashboardChannels::new();
        let state = if let Some(db) = persistence_db.clone() {
            dashboard::DashboardState::with_persistence(&channels, db)
        } else {
            dashboard::DashboardState::new(&channels)
        };
        let port = std::env::var("DASHBOARD_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        // Start dashboard server in a separate async runtime
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(dashboard::start_server(state, port));
        });

        channels
    };

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

        // Process persistence requests
        if let Some(db) = &persistence_db {
            persistence::process_persistence_requests(db, &persistence_channels.rx);
        }

        // Process dashboard requests
        #[cfg(feature = "dashboard")]
        systems::dashboard::system_process_dashboard(&world, &dashboard_channels);

        // Sleep to maintain target FPS
        let elapsed = start.elapsed();
        if elapsed < target_delta {
            thread::sleep(target_delta - elapsed);
        }
    }

    info!("Shutting down...");
    Ok(())
}
