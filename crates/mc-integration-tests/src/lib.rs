//! Integration testing infrastructure for the RGB Minecraft server.
//!
//! This crate provides a test harness that controls both the Rust Minecraft server
//! and a Fabric client via Unix socket IPC to perform integration tests.
//!
//! # Example
//!
//! ```rust,ignore
//! use mc_integration_tests::{IntegrationTest, TestConfig};
//! use std::time::Duration;
//!
//! #[tokio::test]
//! async fn test_login() {
//!     let mut test = IntegrationTest::new(TestConfig::default()).await.unwrap();
//!
//!     test.client().connect("127.0.0.1", test.server().port(), "TestPlayer").await.unwrap();
//!     test.client().wait_for_state("play", Duration::from_secs(30)).await.unwrap();
//!
//!     let state = test.client().get_player_state().await.unwrap();
//!     assert!(state.position.is_some());
//! }
//! ```

pub mod client;
pub mod protocol;
pub mod server;

pub use client::{ClientConfig, FabricClient};
pub use protocol::{ChunkPos, PlayerState, Position, Rotation, TestEvent};
pub use server::{ServerConfig, ServerProcess};

use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

/// Configuration for an integration test
#[derive(Default)]
pub struct TestConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Client configuration
    pub client: ClientConfig,
}

/// An integration test fixture that manages server and client lifecycle
pub struct IntegrationTest {
    server: ServerProcess,
    client: FabricClient,
}

impl IntegrationTest {
    /// Create a new integration test with the given configuration
    ///
    /// # Errors
    /// Returns an error if the server or client fails to start
    pub async fn new(config: TestConfig) -> Result<Self> {
        info!("Starting integration test");

        // Start the server first
        let server = ServerProcess::spawn(config.server).await?;

        // Give server a moment to fully initialize
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Start the client
        let client = FabricClient::spawn(config.client).await?;

        Ok(Self { server, client })
    }

    /// Get a reference to the server
    #[must_use]
    pub fn server(&self) -> &ServerProcess {
        &self.server
    }

    /// Get a mutable reference to the client
    pub fn client(&mut self) -> &mut FabricClient {
        &mut self.client
    }

    /// Teardown the test, killing both server and client
    ///
    /// # Errors
    /// Returns an error if killing the processes fails
    pub async fn teardown(mut self) -> Result<()> {
        info!("Tearing down integration test");
        self.client.kill().await?;
        self.server.kill().await?;
        Ok(())
    }
}

/// Check if integration tests are enabled
///
/// Tests are enabled when the `MC_INTEGRATION_TESTS` environment variable is set.
#[must_use]
pub fn is_enabled() -> bool {
    std::env::var("MC_INTEGRATION_TESTS").is_ok()
}

/// Get the path to the mc-server binary
#[must_use]
pub fn server_binary_path() -> PathBuf {
    // First, determine the workspace root
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok()
        .and_then(|manifest| manifest.parent().and_then(|p| p.parent()).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));

    // If MC_SERVER_BINARY is set, use it (resolve relative paths from workspace root)
    if let Ok(path) = std::env::var("MC_SERVER_BINARY") {
        let path = PathBuf::from(&path);
        if path.is_absolute() {
            return path;
        }
        // Make relative path absolute from workspace root (not current dir)
        return workspace_root.join(path);
    }

    // Default: workspace/target/release/mc-server
    workspace_root.join("target/release/mc-server")
}
