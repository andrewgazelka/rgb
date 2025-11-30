//! Fabric client controller for integration tests.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use eyre::Result;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::UnixStream;
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, info, warn};

use crate::protocol::{
    ChunkPos, ConnectParams, JsonRpcRequest, JsonRpcResponse, MoveToParams, PlayerState, TestEvent,
};

/// Configuration for spawning the Fabric client
pub struct ClientConfig {
    /// Path to the Minecraft game directory
    pub game_dir: PathBuf,
    /// Path to the Java executable
    pub java_path: PathBuf,
    /// Path to the socket file
    pub socket_path: PathBuf,
    /// Startup timeout
    pub startup_timeout: Duration,
    /// Classpath for Minecraft
    pub classpath: String,
    /// Main class to launch
    pub main_class: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("mc-integration-tests");

        Self {
            game_dir: cache_dir.join("minecraft"),
            java_path: PathBuf::from("java"),
            socket_path: PathBuf::from(format!("/tmp/rgb-test-{}.sock", uuid::Uuid::new_v4())),
            startup_timeout: Duration::from_secs(120),
            classpath: String::new(),
            main_class: "net.fabricmc.loader.impl.launch.knot.KnotClient".to_string(),
        }
    }
}

/// A running Fabric client with IPC connection
pub struct FabricClient {
    process: Child,
    writer: Arc<Mutex<BufWriter<tokio::net::unix::OwnedWriteHalf>>>,
    event_rx: mpsc::Receiver<TestEvent>,
    response_rx: mpsc::Receiver<JsonRpcResponse>,
    next_id: AtomicI64,
    collected_events: Vec<TestEvent>,
}

impl FabricClient {
    /// Spawn a new Fabric client and connect to it via Unix socket
    ///
    /// # Errors
    /// Returns an error if the client fails to spawn or connect
    pub async fn spawn(config: ClientConfig) -> Result<Self> {
        info!(
            "Spawning Fabric client with socket at {:?}",
            config.socket_path
        );

        // Ensure game directory exists
        tokio::fs::create_dir_all(&config.game_dir).await?;

        // Clean up any stale socket
        let _ = tokio::fs::remove_file(&config.socket_path).await;

        // Build command
        let mut cmd = Command::new(&config.java_path);
        cmd.arg(format!(
            "-Drgb.test.socket={}",
            config.socket_path.display()
        ))
        .arg("-Djava.awt.headless=false")
        .arg("-cp")
        .arg(&config.classpath)
        .arg(&config.main_class)
        .arg("--gameDir")
        .arg(&config.game_dir)
        .arg("--version")
        .arg("1.21.11-pre3")
        .current_dir(&config.game_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

        let child = cmd.spawn()?;

        // Wait for socket to become available
        let deadline = tokio::time::Instant::now() + config.startup_timeout;
        let socket = loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                eyre::bail!("Timeout waiting for client socket");
            }

            match UnixStream::connect(&config.socket_path).await {
                Ok(socket) => break socket,
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        };

        info!("Connected to Fabric client socket");

        let (read_half, write_half) = socket.into_split();
        let writer = Arc::new(Mutex::new(BufWriter::new(write_half)));

        // Channels for events and responses
        let (event_tx, event_rx) = mpsc::channel(256);
        let (response_tx, response_rx) = mpsc::channel(64);

        // Spawn reader task
        let reader = BufReader::new(read_half);
        tokio::spawn(async move {
            read_messages(reader, event_tx, response_tx).await;
        });

        Ok(Self {
            process: child,
            writer,
            event_rx,
            response_rx,
            next_id: AtomicI64::new(1),
            collected_events: Vec::new(),
        })
    }

    /// Send a JSON-RPC request and wait for response
    async fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = JsonRpcRequest::new(method, params, id);

        let line = serde_json::to_string(&request)?;
        debug!("Sending: {}", line);

        {
            let mut writer = self.writer.lock().await;
            writer.write_all(line.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }

        // Wait for response with matching ID
        let timeout = Duration::from_secs(30);
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            // Drain any events first
            while let Ok(event) = self.event_rx.try_recv() {
                self.collected_events.push(event);
            }

            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                eyre::bail!("Timeout waiting for response to {method}");
            }

            match tokio::time::timeout(remaining, self.response_rx.recv()).await {
                Ok(Some(response)) => {
                    if response.id == Some(id) {
                        return response.into_result();
                    }
                    // Not our response, could be from concurrent request
                    warn!("Received response with unexpected ID: {:?}", response.id);
                }
                Ok(None) => {
                    eyre::bail!("Client connection closed");
                }
                Err(_) => {
                    eyre::bail!("Timeout waiting for response to {method}");
                }
            }
        }
    }

    /// Connect to a Minecraft server
    ///
    /// # Errors
    /// Returns an error if the connection fails
    pub async fn connect(&mut self, host: &str, port: u16, username: &str) -> Result<()> {
        let params = ConnectParams {
            host: host.to_string(),
            port,
            username: username.to_string(),
        };

        self.send_request("connect", Some(serde_json::to_value(params)?))
            .await?;
        Ok(())
    }

    /// Disconnect from the server
    ///
    /// # Errors
    /// Returns an error if disconnection fails
    pub async fn disconnect(&mut self) -> Result<()> {
        self.send_request("disconnect", None).await?;
        Ok(())
    }

    /// Get the current player state
    ///
    /// # Errors
    /// Returns an error if the request fails
    pub async fn get_player_state(&mut self) -> Result<PlayerState> {
        let result = self.send_request("get_player_state", None).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Move the player to a position
    ///
    /// # Errors
    /// Returns an error if the request fails
    pub async fn move_to(&mut self, x: f64, y: f64, z: f64) -> Result<()> {
        let params = MoveToParams { x, y, z };
        self.send_request("move_to", Some(serde_json::to_value(params)?))
            .await?;
        Ok(())
    }

    /// Get all loaded chunks
    ///
    /// # Errors
    /// Returns an error if the request fails
    pub async fn get_loaded_chunks(&mut self) -> Result<Vec<ChunkPos>> {
        let result = self.send_request("get_loaded_chunks", None).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Wait for a specific player state
    ///
    /// # Errors
    /// Returns an error on timeout or if the client disconnects
    pub async fn wait_for_state(&mut self, target_state: &str, timeout: Duration) -> Result<()> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let state = self.get_player_state().await?;
            if state.state == target_state {
                return Ok(());
            }

            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                eyre::bail!(
                    "Timeout waiting for state '{}', current state: '{}'",
                    target_state,
                    state.state
                );
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Wait for chunks to load
    ///
    /// # Errors
    /// Returns an error on timeout
    pub async fn wait_for_chunks(
        &mut self,
        count: usize,
        timeout: Duration,
    ) -> Result<Vec<ChunkPos>> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let chunks = self.get_loaded_chunks().await?;
            if chunks.len() >= count {
                return Ok(chunks);
            }

            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                eyre::bail!(
                    "Timeout waiting for {} chunks, only have {}",
                    count,
                    chunks.len()
                );
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Wait for a specific event type
    ///
    /// # Errors
    /// Returns an error on timeout
    pub async fn wait_for_event<F>(&mut self, predicate: F, timeout: Duration) -> Result<TestEvent>
    where
        F: Fn(&TestEvent) -> bool,
    {
        // Check already collected events
        for (i, event) in self.collected_events.iter().enumerate() {
            if predicate(event) {
                return Ok(self.collected_events.remove(i));
            }
        }

        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                eyre::bail!("Timeout waiting for event");
            }

            match tokio::time::timeout(remaining, self.event_rx.recv()).await {
                Ok(Some(event)) => {
                    if predicate(&event) {
                        return Ok(event);
                    }
                    self.collected_events.push(event);
                }
                Ok(None) => {
                    eyre::bail!("Client connection closed");
                }
                Err(_) => {
                    eyre::bail!("Timeout waiting for event");
                }
            }
        }
    }

    /// Get all collected events (events received but not yet consumed)
    #[must_use]
    pub fn events(&self) -> &[TestEvent] {
        &self.collected_events
    }

    /// Ping the client to check connection
    ///
    /// # Errors
    /// Returns an error if the ping fails
    pub async fn ping(&mut self) -> Result<()> {
        self.send_request("ping", None).await?;
        Ok(())
    }

    /// Kill the client process
    ///
    /// # Errors
    /// Returns an error if killing fails
    pub async fn kill(&mut self) -> Result<()> {
        self.process.kill().await?;
        Ok(())
    }
}

impl Drop for FabricClient {
    fn drop(&mut self) {
        let _ = self.process.start_kill();
    }
}

/// Background task to read messages from the socket
async fn read_messages(
    mut reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    event_tx: mpsc::Sender<TestEvent>,
    response_tx: mpsc::Sender<JsonRpcResponse>,
) {
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                debug!("Socket closed");
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                debug!("Received: {}", line);

                // Try to parse as response first
                if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(line) {
                    if response.id.is_some() {
                        let _ = response_tx.send(response).await;
                        continue;
                    }
                }

                // Try to parse as notification (event)
                if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(line) {
                    if let Some(event) = crate::protocol::parse_event(&request) {
                        let _ = event_tx.send(event).await;
                    }
                }
            }
            Err(e) => {
                warn!("Error reading from socket: {}", e);
                break;
            }
        }
    }
}
