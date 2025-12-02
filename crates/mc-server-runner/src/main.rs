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

    // TODO: Refactor these components to use relations instead of VecDeque/channels
    // For now, they work as regular components but should be migrated to:
    // - PacketBuffer: Each packet as entity with (PacketData, Pair::<PendingFor>(connection))
    // - Network channels: Use event system or external channel management

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

    // Start dashboard server if feature enabled
    #[cfg(feature = "dashboard")]
    {
        use rgb_ecs_introspect::{IntrospectChannels, IntrospectIngress, IntrospectRegistry};

        let mut registry = IntrospectRegistry::new();
        // Register components for introspection (must be done after world creates them)
        registry.register::<Position>(&world);
        registry.register::<Rotation>(&world);
        registry.register::<Name>(&world);
        registry.register::<ServerConfig>(&world);
        registry.register::<WorldTime>(&world);
        registry.register::<ChunkData>(&world);

        let registry = Arc::new(registry);
        let introspect_channels = IntrospectChannels::default();

        // Add ingress component so systems can process requests
        world.insert(
            Entity::WORLD,
            IntrospectIngress {
                rx: introspect_channels.request_rx.clone(),
                registry: registry.clone(),
            },
        );

        let dashboard_state = dashboard::DashboardState::new(&introspect_channels, registry);
        let dashboard_port = std::env::var("DASHBOARD_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080u16);

        // Spawn dashboard server in background thread with its own tokio runtime
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime")
                .block_on(async move {
                    dashboard::start_server(dashboard_state, dashboard_port).await;
                });
        });

        info!("Dashboard server starting on port {}", dashboard_port);
    }

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
