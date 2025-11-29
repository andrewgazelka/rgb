//! Integration test for the login flow.
//!
//! This test verifies that a Fabric client can successfully:
//! 1. Connect to the server
//! 2. Complete the login sequence (handshake -> login -> config -> play)
//! 3. Receive an entity ID and position
//!
//! NOTE: These tests require a full Fabric Minecraft client setup.
//! They are skipped by default unless MC_FABRIC_CLIENT is set.

use std::time::Duration;

use mc_integration_tests::{IntegrationTest, TestConfig, TestEvent, is_enabled};

fn fabric_client_available() -> bool {
    // These tests require Fabric client setup which is complex
    // Skip unless explicitly enabled
    std::env::var("MC_FABRIC_CLIENT").is_ok()
}

#[tokio::test]
async fn test_login_flow() {
    if !is_enabled() {
        eprintln!("Skipping integration test (set MC_INTEGRATION_TESTS=1 to enable)");
        return;
    }
    if !fabric_client_available() {
        eprintln!("Skipping test (requires Fabric client, set MC_FABRIC_CLIENT to enable)");
        return;
    }

    let config = TestConfig::default();
    let mut test = IntegrationTest::new(config)
        .await
        .expect("Failed to setup integration test");

    // Connect to the server
    let port = test.server().port();
    test.client()
        .connect("127.0.0.1", port, "TestPlayer")
        .await
        .expect("Failed to connect");

    // Wait for play state
    test.client()
        .wait_for_state("play", Duration::from_secs(30))
        .await
        .expect("Failed to reach play state");

    // Verify player state
    let state = test
        .client()
        .get_player_state()
        .await
        .expect("Failed to get player state");

    assert!(state.connected, "Player should be connected");
    assert_eq!(state.state, "play", "Player should be in play state");
    assert!(state.entity_id.is_some(), "Player should have an entity ID");
    assert!(state.position.is_some(), "Player should have a position");

    if let Some(pos) = &state.position {
        eprintln!("Player position: ({}, {}, {})", pos.x, pos.y, pos.z);
    }

    // Disconnect
    test.client()
        .disconnect()
        .await
        .expect("Failed to disconnect");

    test.teardown().await.expect("Failed to teardown");
}

#[tokio::test]
async fn test_chunk_loading() {
    if !is_enabled() {
        eprintln!("Skipping integration test (set MC_INTEGRATION_TESTS=1 to enable)");
        return;
    }
    if !fabric_client_available() {
        eprintln!("Skipping test (requires Fabric client, set MC_FABRIC_CLIENT to enable)");
        return;
    }

    let config = TestConfig::default();
    let mut test = IntegrationTest::new(config)
        .await
        .expect("Failed to setup integration test");

    // Connect and wait for play
    let port = test.server().port();
    test.client()
        .connect("127.0.0.1", port, "ChunkTestPlayer")
        .await
        .expect("Failed to connect");

    test.client()
        .wait_for_state("play", Duration::from_secs(30))
        .await
        .expect("Failed to reach play state");

    // Wait for spawn chunks (7x7 = 49 chunks around spawn)
    let chunks = test
        .client()
        .wait_for_chunks(49, Duration::from_secs(60))
        .await
        .expect("Failed to load spawn chunks");

    eprintln!("Loaded {} chunks", chunks.len());
    assert!(chunks.len() >= 49, "Should have loaded at least 49 chunks");

    // Verify we have the origin chunk
    let has_origin = chunks.iter().any(|c| c.x == 0 && c.z == 0);
    assert!(has_origin, "Should have loaded the origin chunk (0, 0)");

    test.teardown().await.expect("Failed to teardown");
}

#[tokio::test]
async fn test_player_events() {
    if !is_enabled() {
        eprintln!("Skipping integration test (set MC_INTEGRATION_TESTS=1 to enable)");
        return;
    }
    if !fabric_client_available() {
        eprintln!("Skipping test (requires Fabric client, set MC_FABRIC_CLIENT to enable)");
        return;
    }

    let config = TestConfig::default();
    let mut test = IntegrationTest::new(config)
        .await
        .expect("Failed to setup integration test");

    let port = test.server().port();
    test.client()
        .connect("127.0.0.1", port, "EventTestPlayer")
        .await
        .expect("Failed to connect");

    // Wait for login success event
    let login_event = test
        .client()
        .wait_for_event(
            |e| matches!(e, TestEvent::LoginSuccess { .. }),
            Duration::from_secs(30),
        )
        .await
        .expect("Never received login success event");

    if let TestEvent::LoginSuccess { uuid, username } = login_event {
        eprintln!("Login success: {} ({})", username, uuid);
        assert_eq!(username, "EventTestPlayer");
    }

    // Wait for play state event
    let play_event = test
        .client()
        .wait_for_event(
            |e| matches!(e, TestEvent::PlayState { .. }),
            Duration::from_secs(30),
        )
        .await
        .expect("Never received play state event");

    if let TestEvent::PlayState { entity_id } = play_event {
        eprintln!("Entered play state with entity ID: {}", entity_id);
        assert!(entity_id > 0, "Entity ID should be positive");
    }

    test.teardown().await.expect("Failed to teardown");
}
