//! Server process management for integration tests.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{debug, info};

/// Configuration for spawning the Minecraft server
pub struct ServerConfig {
    /// Path to the mc-server binary
    pub binary_path: PathBuf,
    /// Timeout for server startup
    pub startup_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            binary_path: PathBuf::from("target/release/mc-server"),
            startup_timeout: Duration::from_secs(30),
        }
    }
}

/// A running Minecraft server process
pub struct ServerProcess {
    process: Child,
    port: u16,
}

impl ServerProcess {
    /// Spawn a new server process with auto-assigned port
    ///
    /// # Errors
    /// Returns an error if the server fails to spawn or doesn't start within the timeout
    pub async fn spawn(config: ServerConfig) -> Result<Self> {
        info!("Spawning server with auto-assigned port");

        let mut cmd = Command::new(&config.binary_path);
        cmd.env("MC_PORT", "0") // Port 0 = auto-assign
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn()?;

        let mut actual_port: Option<u16> = None;

        // Wait for server to be ready by monitoring stdout for startup message
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            let deadline = tokio::time::Instant::now() + config.startup_timeout;

            loop {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                if remaining.is_zero() {
                    anyhow::bail!("Server startup timeout");
                }

                match tokio::time::timeout(remaining, lines.next_line()).await {
                    Ok(Ok(Some(line))) => {
                        debug!("Server: {}", line);

                        // Parse SERVER_PORT=XXXXX from output
                        if let Some(port_str) = line.strip_suffix(|_| true).and_then(|_| {
                            line.split("SERVER_PORT=").nth(1).and_then(|s| {
                                s.split_whitespace()
                                    .next()
                                    .or_else(|| s.split('\x1b').next())
                            })
                        }) {
                            if let Ok(port) = port_str.trim().parse::<u16>() {
                                actual_port = Some(port);
                                info!("Server assigned port {}", port);
                            }
                        }

                        // Also check for simple format
                        if actual_port.is_none() && line.contains("SERVER_PORT=") {
                            // Extract port from line like "... SERVER_PORT=12345 ..." or ANSI colored
                            for part in line.split(|c: char| !c.is_ascii_digit()) {
                                if part.len() >= 4 && part.len() <= 5 {
                                    if let Ok(port) = part.parse::<u16>() {
                                        if port > 1024 {
                                            actual_port = Some(port);
                                            info!("Server assigned port {}", port);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if line.contains("listening") {
                            if actual_port.is_some() {
                                info!("Server started successfully");
                                break;
                            }
                            // Try to extract port from "listening on 0.0.0.0:XXXXX"
                            if let Some(addr) = line.split("listening on ").nth(1) {
                                if let Some(port_str) = addr.split(':').next_back() {
                                    // Remove ANSI codes if present
                                    let clean = port_str
                                        .chars()
                                        .take_while(|c| c.is_ascii_digit())
                                        .collect::<String>();
                                    if let Ok(port) = clean.parse::<u16>() {
                                        actual_port = Some(port);
                                        info!("Server started on port {}", port);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Ok(Ok(None)) => {
                        anyhow::bail!("Server process ended unexpectedly");
                    }
                    Ok(Err(e)) => {
                        anyhow::bail!("Error reading server output: {e}");
                    }
                    Err(_) => {
                        anyhow::bail!("Server startup timeout");
                    }
                }
            }
        }

        let port = actual_port.ok_or_else(|| anyhow::anyhow!("Could not determine server port"))?;

        // Give server a moment to fully initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(Self {
            process: child,
            port,
        })
    }

    /// Get the port the server is listening on
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Kill the server process
    ///
    /// # Errors
    /// Returns an error if killing the process fails
    pub async fn kill(&mut self) -> Result<()> {
        self.process.kill().await?;
        Ok(())
    }

    /// Wait for the server process to exit
    ///
    /// # Errors
    /// Returns an error if waiting fails
    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        Ok(self.process.wait().await?)
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        // Best-effort kill
        let _ = self.process.start_kill();
    }
}
