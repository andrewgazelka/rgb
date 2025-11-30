//! Minecraft server runner with hot-reloadable module support
//!
//! This binary:
//! 1. Sets up network channels and singletons
//! 2. Loads modules from the `modules/` directory
//! 3. Runs the game loop with hot-reload support
//!
//! Commands:
//! - `r` or `reload` - Reload all modules
//! - `l` or `list` - List loaded modules
//! - `q` or `quit` - Quit the server
//! - `help` - Show help

use std::collections::HashMap;
use std::io::{self, Cursor, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use bytes::Bytes;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, execute};
use flecs_ecs::prelude::*;
use mc_protocol::read_varint;
use module_loader::ModuleLoader;
use module_network_components::{
    DisconnectEvent, DisconnectIngress, IncomingPacket, NetworkChannels, NetworkEgress,
    NetworkIngress, OutgoingPacket,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Active connections map (connection_id -> sender for that connection)
type ConnectionMap = Arc<RwLock<HashMap<u64, tokio::sync::mpsc::Sender<Bytes>>>>;

/// Commands that can be sent from the input thread
enum Command {
    Reload,
    List,
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
                .add_directive("module_loader=info".parse()?)
                .add_directive("mc_server_lib=debug".parse()?),
        )
        .init();

    info!(
        "Starting Minecraft server with hot-reloadable modules (version {})",
        mc_data::PROTOCOL_NAME
    );

    // Determine modules directory
    let modules_dir = std::env::var("MODULES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("modules"));

    info!("Modules directory: {}", modules_dir.display());

    // Create network channels for ECS <-> async bridge
    let channels = NetworkChannels::new();

    // Create empty Flecs world - modules will register everything
    let world = World::new();

    // Set up network channel singletons (these are needed by modules)
    world
        .component::<NetworkIngress>()
        .add_trait::<flecs::Singleton>();
    world
        .component::<NetworkEgress>()
        .add_trait::<flecs::Singleton>();
    world
        .component::<DisconnectIngress>()
        .add_trait::<flecs::Singleton>();

    world.set(NetworkIngress {
        rx: channels.ingress_rx.clone(),
    });
    world.set(NetworkEgress {
        tx: channels.egress_tx.clone(),
    });
    world.set(DisconnectIngress {
        rx: channels.disconnect_rx.clone(),
    });

    // Create module loader
    let mut loader = ModuleLoader::new(&modules_dir);

    // Load all modules from the directory
    if let Err(e) = loader.load_all(&world) {
        error!("Failed to load modules: {}", e);
    }

    // Start watching for file changes
    if let Err(e) = loader.start_watching() {
        error!("Failed to start file watcher: {}", e);
    }

    info!("Loaded modules: {:?}", loader.loaded_modules());

    // Configuration
    let rest_port: u16 = std::env::var("REST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(27750);

    let target_fps: f32 = std::env::var("TARGET_FPS")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(20.0);

    let mc_port: u16 = std::env::var("MC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(25565);

    info!(
        "Flecs Explorer available at https://www.flecs.dev/explorer (connect to localhost:{})",
        rest_port
    );

    // Enable REST API for Flecs Explorer
    world.set(flecs::rest::Rest::default());
    world.import::<flecs_ecs::addons::stats::Stats>();

    // Start async network runtime in background
    let ingress_tx = channels.ingress_tx.clone();
    let egress_rx = channels.egress_rx.clone();
    let disconnect_tx = channels.disconnect_tx.clone();

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(run_network(mc_port, ingress_tx, egress_rx, disconnect_tx));
    });

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

        // Poll for module changes (file watcher)
        let reloaded = loader.poll_reload(&world);
        if reloaded > 0 {
            clear_line();
            info!("Reloaded {} module(s)", reloaded);
            print_prompt();
        }

        // Check for commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            clear_line();
            match cmd {
                Command::Reload => {
                    info!("Manual reload requested");
                    let count = loader.reload_all(&world);
                    info!("Reloaded {} module(s)", count);
                }
                Command::List => {
                    let modules = loader.loaded_modules();
                    info!("Loaded modules ({}):", modules.len());
                    for name in modules {
                        info!("  - {}", name);
                    }
                }
                Command::Quit => {
                    info!("Shutting down...");
                    running = false;
                }
                Command::Help => {
                    println!("\r\nCommands:");
                    println!("  r, reload  - Reload all modules");
                    println!("  l, list    - List loaded modules");
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

    Ok(())
}

async fn run_network(
    port: u16,
    ingress_tx: crossbeam_channel::Sender<IncomingPacket>,
    egress_rx: crossbeam_channel::Receiver<OutgoingPacket>,
    disconnect_tx: crossbeam_channel::Sender<DisconnectEvent>,
) {
    let connections: ConnectionMap = Arc::new(RwLock::new(HashMap::new()));
    let connections_clone = connections.clone();

    // Spawn egress handler
    let rt_handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        while let Ok(packet) = egress_rx.recv() {
            let connections = connections_clone.clone();
            let data = packet.data;
            let conn_id = packet.connection_id;
            rt_handle.block_on(async move {
                let connections = connections.read().await;
                if let Some(tx) = connections.get(&conn_id) {
                    let _ = tx.send(data).await;
                }
            });
        }
    });

    // Start TCP listener
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    let actual_port = listener.local_addr().unwrap().port();
    info!("SERVER_PORT={}", actual_port);
    info!("Minecraft server listening on 0.0.0.0:{}", actual_port);

    let mut next_conn_id: u64 = 1;

    loop {
        let (stream, addr) = listener.accept().await.expect("Failed to accept");
        info!("Connection from {}", addr);

        let conn_id = next_conn_id;
        next_conn_id += 1;

        let ingress_tx = ingress_tx.clone();
        let disconnect_tx = disconnect_tx.clone();
        let connections = connections.clone();

        tokio::spawn(async move {
            let (tx, rx) = tokio::sync::mpsc::channel::<Bytes>(256);

            {
                let mut conns = connections.write().await;
                conns.insert(conn_id, tx);
            }

            let result = handle_connection(stream, conn_id, ingress_tx, rx).await;

            {
                let mut conns = connections.write().await;
                conns.remove(&conn_id);
            }

            let _ = disconnect_tx.send(DisconnectEvent {
                connection_id: conn_id,
            });

            if let Err(e) = result {
                debug!("Connection {} closed: {}", conn_id, e);
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    conn_id: u64,
    ingress_tx: crossbeam_channel::Sender<IncomingPacket>,
    mut egress_rx: tokio::sync::mpsc::Receiver<Bytes>,
) -> eyre::Result<()> {
    let (mut reader, mut writer) = stream.into_split();

    let writer_handle = tokio::spawn(async move {
        while let Some(data) = egress_rx.recv().await {
            if writer.write_all(&data).await.is_err() {
                break;
            }
            if writer.flush().await.is_err() {
                break;
            }
        }
    });

    loop {
        let Ok(length) = read_varint_async(&mut reader).await else {
            break;
        };

        if length <= 0 {
            continue;
        }

        let mut data = vec![0u8; length as usize];
        if reader.read_exact(&mut data).await.is_err() {
            break;
        }

        let mut cursor = Cursor::new(&data);
        let packet_id = read_varint(&mut cursor)?;
        let remaining = data[cursor.position() as usize..].to_vec();

        let _ = ingress_tx.send(IncomingPacket {
            connection_id: conn_id,
            packet_id,
            data: remaining.into(),
        });
    }

    writer_handle.abort();
    Ok(())
}

async fn read_varint_async<R: AsyncReadExt + Unpin>(reader: &mut R) -> eyre::Result<i32> {
    let mut result = 0i32;
    let mut shift = 0;
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf).await?;
        let byte = buf[0];
        result |= ((byte & 0x7F) as i32) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            eyre::bail!("VarInt too large");
        }
    }
    Ok(result)
}

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
    execute!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine)
    )
    .ok();
}

fn update_title(tick: u64) {
    print!("\x1b]0;MC Server - Tick: {tick}\x07");
    io::stdout().flush().ok();
}
