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

use std::io::{self, Write as _};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, execute};
use rgb_ecs::prelude::*;
use tracing::info;

use rgb_event::EventPlugin;

use crate::components::*;
use crate::network::NetworkChannels;

/// Commands from input thread
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
                .add_directive("mc_server_runner=info".parse()?),
        )
        .init();

    info!("Starting Minecraft server with RGB ECS");

    // Create ECS world with plugins
    let mut world = World::new();

    // Add event system plugin
    world.add_plugin(EventPlugin);

    // Initialize global state on Entity::WORLD (auto-registers component types)
    world.insert(Entity::WORLD, WorldTime::default());
    world.insert(Entity::WORLD, TpsTracker::default());
    world.insert(Entity::WORLD, EntityIdCounter::default());
    world.insert(Entity::WORLD, ConnectionIndex::default());
    world.insert(Entity::WORLD, ChunkIndex::default());

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

    // Set up command input channel
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();

    // Spawn input thread
    thread::spawn(move || {
        input_thread(cmd_tx);
    });

    // Enable raw mode for keyboard input
    terminal::enable_raw_mode().ok();
    print_prompt();

    // Run game loop at 20 TPS
    let target_fps: f32 = std::env::var("TARGET_FPS")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(20.0);
    let target_delta = Duration::from_secs_f32(1.0 / target_fps);
    let mut tick: u64 = 0;
    let mut running = true;
    let mut last_tick = Instant::now();

    while running {
        let start = Instant::now();

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

        // Calculate delta time
        let delta_time = last_tick.elapsed().as_secs_f32();
        last_tick = Instant::now();

        // Run all systems
        systems::tick(&mut world, delta_time);
        tick += 1;

        // Update title with tick count
        if tick % 20 == 0 {
            update_title(tick);
        }

        // Sleep to maintain target FPS
        let elapsed = start.elapsed();
        if elapsed < target_delta {
            thread::sleep(target_delta - elapsed);
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
