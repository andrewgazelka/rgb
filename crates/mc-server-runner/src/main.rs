//! Minecraft server runner with Flecs ECS
//!
//! Commands:
//! - `q` or `quit` - Quit the server
//! - `help` - Show help

use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, execute};
use flecs_ecs::prelude::*;
use tracing::info;

/// Commands that can be sent from the input thread
enum Command {
    Quit,
    Help,
    Unknown(String),
}

fn main() -> eyre::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mc_server_runner=info".parse()?)
                .add_directive("module_listener=info".parse()?)
                .add_directive("module_handshake=info".parse()?)
                .add_directive("module_login=info".parse()?)
                .add_directive("module_config=info".parse()?)
                .add_directive("module_play=info".parse()?)
                .add_directive("module_chunk=info".parse()?),
        )
        .init();

    info!("Starting Minecraft server");

    // Create Flecs world
    let world = World::new();

    // Persistence layer - must be initialized before importing modules that use .persist()
    persist::init::<module_login_components::Uuid>(&world, "./world_data");

    // Import all modules statically
    // Order matters: components first, then systems that depend on them
    world.import::<module_network_components::NetworkComponentsModule>();
    world.import::<module_time_components::TimeComponentsModule>();
    world.import::<module_login_components::LoginComponentsModule>();
    world.import::<module_chunk_components::ChunkComponentsModule>();

    // Network layer
    world.import::<module_listener::ListenerModule>();
    world.import::<module_network_systems::NetworkSystemsModule>();

    // Protocol handling
    world.import::<module_handshake::HandshakeModule>();
    world.import::<module_login::LoginModule>();
    world.import::<module_config::ConfigurationModule>();
    world.import::<module_play::PlayModule>();
    world.import::<module_chunk::ChunkModule>();
    world.import::<module_time::TimeModule>();

    // Generate spawn chunks
    module_chunk::generate_spawn_chunks(&world, 8);

    info!("Loaded all modules");

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
    thread::spawn(move || {
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

        // Check for commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            clear_line();
            match cmd {
                Command::Quit => {
                    info!("Shutting down...");
                    running = false;
                }
                Command::Help => {
                    info!("\r\nCommands:");
                    info!("  q, quit    - Quit the server");
                    info!("  help       - Show this help");
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
        if tick.is_multiple_of(20) {
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

    Ok(())
}

#[allow(clippy::print_stdout)]
fn input_thread(tx: mpsc::Sender<Command>) {
    let mut input_buffer = String::new();

    loop {
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = event::read() {
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
                        print!("{c}");
                        io::stdout().flush().ok();
                    }
                    KeyCode::Backspace => {
                        if input_buffer.pop().is_some() {
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
        "q" | "quit" | "exit" => Command::Quit,
        "help" | "h" | "?" => Command::Help,
        other => Command::Unknown(other.to_string()),
    }
}

#[allow(clippy::print_stdout)]
fn print_prompt() {
    print!("\r> ");
    io::stdout().flush().ok();
}

fn clear_line() {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine)
    )
    .ok();
}

#[allow(clippy::print_stdout)]
fn update_title(tick: u64) {
    print!("\x1b]0;MC Server - Tick: {tick}\x07");
    io::stdout().flush().ok();
}
