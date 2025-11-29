//! Minecraft server runner with hot-reloadable plugin support
//!
//! This binary loads Flecs modules from dylib plugins in the `plugins/` directory
//! and supports hot-reloading when plugins are modified.

use std::path::PathBuf;
use std::time::Duration;

use flecs_ecs::prelude::*;
use plugin_loader::PluginLoader;
use tracing::{error, info};

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_server_runner=info".parse()?)
                .add_directive("plugin_loader=info".parse()?),
        )
        .init();

    info!("Starting Minecraft server with hot-reloadable plugins");

    // Determine plugins directory
    let plugins_dir = std::env::var("PLUGINS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("plugins"));

    info!("Plugins directory: {}", plugins_dir.display());

    // Create the Flecs world
    let world = World::new();

    // Create plugin loader
    let mut loader = PluginLoader::new(&plugins_dir);

    // Load all plugins from the directory
    if let Err(e) = loader.load_all(&world) {
        error!("Failed to load plugins: {}", e);
    }

    // Start watching for file changes
    if let Err(e) = loader.start_watching() {
        error!("Failed to start file watcher: {}", e);
    }

    info!("Loaded plugins: {:?}", loader.loaded_plugins());

    // Configuration
    let rest_port: u16 = std::env::var("REST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(27750);

    let target_fps: f32 = std::env::var("TARGET_FPS")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(20.0);

    info!(
        "Flecs Explorer available at https://www.flecs.dev/explorer (connect to localhost:{})",
        rest_port
    );

    // Run game loop with hot-reload polling
    // We use a manual loop instead of world.app().run() so we can poll for reloads
    let target_delta = Duration::from_secs_f32(1.0 / target_fps);

    // Enable REST API for Flecs Explorer
    world.set(flecs::rest::Rest::default());
    world.import::<flecs_ecs::addons::stats::Stats>();

    loop {
        let start = std::time::Instant::now();

        // Poll for plugin changes
        let reloaded = loader.poll_reload(&world);
        if reloaded > 0 {
            info!("Reloaded {} plugin(s)", reloaded);
            info!("Loaded plugins: {:?}", loader.loaded_plugins());
        }

        // Progress the world
        world.progress();

        // Sleep to maintain target FPS
        let elapsed = start.elapsed();
        if elapsed < target_delta {
            std::thread::sleep(target_delta - elapsed);
        }
    }
}
