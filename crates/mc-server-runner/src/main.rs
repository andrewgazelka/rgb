//! Minecraft server runner with hot-reloadable plugin support
//!
//! This binary loads Flecs modules from dylib plugins in the `plugins/` directory
//! and supports hot-reloading when plugins are modified.
//!
//! Commands:
//! - `r` or `reload` - Reload all plugins
//! - `l` or `list` - List loaded plugins
//! - `q` or `quit` - Quit the server
//! - `help` - Show help

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, execute};
use flecs_ecs::prelude::*;
use plugin_loader::PluginLoader;
use tracing::{error, info};

/// Commands that can be sent from the input thread
enum Command {
    Reload,
    List,
    Quit,
    Help,
    Unknown(String),
}

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

    // Enable REST API for Flecs Explorer
    world.set(flecs::rest::Rest::default());
    world.import::<flecs_ecs::addons::stats::Stats>();

    // Set up command input channel
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();

    // Spawn input thread
    let input_handle = thread::spawn(move || {
        input_thread(cmd_tx);
    });

    // Enable raw mode for keyboard input
    terminal::enable_raw_mode().ok();

    // Print help
    print_prompt();

    // Run game loop
    let target_delta = Duration::from_secs_f32(1.0 / target_fps);
    let mut tick: u64 = 0;
    let mut running = true;

    while running {
        let start = std::time::Instant::now();

        // Poll for plugin changes (file watcher)
        let reloaded = loader.poll_reload(&world);
        if reloaded > 0 {
            clear_line();
            info!("Reloaded {} plugin(s)", reloaded);
            print_prompt();
        }

        // Check for commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            clear_line();
            match cmd {
                Command::Reload => {
                    info!("Manual reload requested");
                    let count = loader.reload_all(&world);
                    info!("Reloaded {} plugin(s)", count);
                }
                Command::List => {
                    let plugins = loader.loaded_plugins();
                    info!("Loaded plugins ({}):", plugins.len());
                    for name in plugins {
                        info!("  - {}", name);
                    }
                }
                Command::Quit => {
                    info!("Shutting down...");
                    running = false;
                }
                Command::Help => {
                    println!("\r\nCommands:");
                    println!("  r, reload  - Reload all plugins");
                    println!("  l, list    - List loaded plugins");
                    println!("  q, quit    - Quit the server");
                    println!("  help       - Show this help");
                }
                Command::Unknown(s) => {
                    if !s.is_empty() {
                        info!("Unknown command: '{}'. Type 'help' for commands.", s);
                    }
                }
            }
            print_prompt();
        }

        // Progress the world
        world.progress();
        tick += 1;

        // Update title with tick count
        if tick % 20 == 0 {
            update_title(tick);
        }

        // Sleep to maintain target FPS
        let elapsed = start.elapsed();
        if elapsed < target_delta {
            std::thread::sleep(target_delta - elapsed);
        }
    }

    // Cleanup
    terminal::disable_raw_mode().ok();
    loader.unload_all(&world);

    // Wait for input thread (it will exit when it detects we're done)
    drop(input_handle);

    Ok(())
}

fn input_thread(tx: mpsc::Sender<Command>) {
    let mut input_buffer = String::new();

    loop {
        // Poll for keyboard events with a small timeout
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = event::read() {
                // Handle Ctrl+C
                if key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && key_event.code == KeyCode::Char('c')
                {
                    let _ = tx.send(Command::Quit);
                    break;
                }

                match key_event.code {
                    KeyCode::Enter => {
                        let cmd = parse_command(&input_buffer);
                        let is_quit = matches!(cmd, Command::Quit);
                        let _ = tx.send(cmd);
                        input_buffer.clear();
                        if is_quit {
                            break;
                        }
                    }
                    KeyCode::Char(c) => {
                        input_buffer.push(c);
                        // Echo character
                        print!("{c}");
                        io::stdout().flush().ok();
                    }
                    KeyCode::Backspace => {
                        if input_buffer.pop().is_some() {
                            // Move cursor back, print space, move back again
                            print!("\x08 \x08");
                            io::stdout().flush().ok();
                        }
                    }
                    KeyCode::Esc => {
                        input_buffer.clear();
                        clear_line();
                        print_prompt();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn parse_command(input: &str) -> Command {
    match input.trim().to_lowercase().as_str() {
        "r" | "reload" => Command::Reload,
        "l" | "list" => Command::List,
        "q" | "quit" | "exit" => Command::Quit,
        "help" | "h" | "?" => Command::Help,
        other => Command::Unknown(other.to_string()),
    }
}

fn print_prompt() {
    print!("\r> ");
    io::stdout().flush().ok();
}

fn clear_line() {
    let mut stdout = io::stdout();
    execute!(stdout, cursor::MoveToColumn(0), terminal::Clear(ClearType::CurrentLine)).ok();
}

fn update_title(tick: u64) {
    // Set terminal title with tick count
    print!("\x1b]0;MC Server - Tick: {tick}\x07");
    io::stdout().flush().ok();
}
