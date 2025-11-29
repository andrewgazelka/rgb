//! Smoke test for the server - verifies it starts and accepts connections.
//!
//! This test doesn't require a Fabric client, just checks the server basics.

use std::time::Duration;

use mc_integration_tests::{ServerConfig, ServerProcess, is_enabled, server_binary_path};
use tokio::net::TcpStream;

#[tokio::test]
async fn test_server_starts_and_accepts_connections() {
    if !is_enabled() {
        eprintln!("Skipping integration test (set MC_INTEGRATION_TESTS=1 to enable)");
        return;
    }

    let binary = server_binary_path();
    if !binary.exists() {
        eprintln!(
            "Server binary not found at {:?}, run `cargo build -p mc-server --release` first",
            binary
        );
        return;
    }

    let config = ServerConfig {
        binary_path: binary,
        startup_timeout: Duration::from_secs(30),
    };

    let server = ServerProcess::spawn(config)
        .await
        .expect("Failed to start server");

    eprintln!("Server started on port {}", server.port());

    // Try to connect via TCP
    let stream = TcpStream::connect(format!("127.0.0.1:{}", server.port()))
        .await
        .expect("Failed to connect to server");

    eprintln!("TCP connection established");

    // Connection established - server is accepting connections
    drop(stream);

    eprintln!("Server smoke test passed!");
}
