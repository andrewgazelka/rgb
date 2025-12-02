//! ECS Dashboard web server for introspection.
//!
//! Provides REST API endpoints for inspecting ECS state from a web dashboard.
//! Uses a channel-based approach to safely communicate with the Flecs world.

use std::time::Duration;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use crossbeam_channel::{Receiver, Sender, bounded};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

// ============================================================================
// Request/Response Types for Channel Communication
// ============================================================================

/// Request types that can be sent to the game loop.
pub enum DashboardRequest {
    GetWorld {
        response: Sender<WorldInfo>,
    },
    ListEntities {
        limit: usize,
        offset: usize,
        response: Sender<ListEntitiesResponse>,
    },
    GetEntity {
        id: u64,
        response: Sender<Option<EntityDetails>>,
    },
    ListPlayers {
        response: Sender<Vec<PlayerInfo>>,
    },
    ListChunks {
        response: Sender<Vec<ChunkInfo>>,
    },
    GetEntityHistory {
        id: u64,
        response: Sender<Vec<HistoryEntryInfo>>,
    },
}

#[derive(Serialize, Clone)]
pub struct WorldInfo {
    pub entity_count: usize,
    pub archetype_count: usize,
    pub component_count: usize,
    pub globals: serde_json::Value,
}

#[derive(Serialize, Clone)]
pub struct EntitySummary {
    pub id: u64,
    pub name: Option<String>,
    pub components: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct ListEntitiesResponse {
    pub entities: Vec<EntitySummary>,
    pub total: usize,
}

#[derive(Serialize, Clone)]
pub struct EntityDetails {
    pub id: u64,
    pub name: Option<String>,
    pub components: Vec<ComponentValue>,
}

#[derive(Serialize, Clone)]
pub struct ComponentValue {
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Serialize, Clone)]
pub struct ChunkInfo {
    pub x: i32,
    pub z: i32,
}

#[derive(Serialize, Clone)]
pub struct PlayerInfo {
    pub entity_id: u64,
    pub name: Option<String>,
    pub uuid: Option<String>,
    pub position: Option<PositionInfo>,
    pub game_mode: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct PositionInfo {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Clone)]
pub struct HistoryEntryInfo {
    pub tick: u64,
    pub component_id: u64,
    pub data_size: usize,
}

// ============================================================================
// Dashboard Channels
// ============================================================================

/// Channels for dashboard communication with the game loop.
pub struct DashboardChannels {
    /// Sender for dashboard to send requests to game loop.
    pub request_tx: Sender<DashboardRequest>,
    /// Receiver for game loop to receive requests from dashboard.
    pub request_rx: Receiver<DashboardRequest>,
}

impl DashboardChannels {
    /// Create a new pair of dashboard channels.
    pub fn new() -> Self {
        let (request_tx, request_rx) = bounded(64);
        Self {
            request_tx,
            request_rx,
        }
    }
}

impl Default for DashboardChannels {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Dashboard State (for Axum)
// ============================================================================

/// Shared state for the dashboard API.
#[derive(Clone)]
pub struct DashboardState {
    /// Channel to send requests to the game loop.
    request_tx: Sender<DashboardRequest>,
}

impl DashboardState {
    /// Create dashboard state from channels.
    pub fn new(channels: &DashboardChannels) -> Self {
        Self {
            request_tx: channels.request_tx.clone(),
        }
    }
}

/// Request timeout for game loop operations.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

// ============================================================================
// Router
// ============================================================================

/// Create the dashboard router with all endpoints.
pub fn create_router(state: DashboardState) -> Router {
    let cors = CorsLayer::permissive();

    Router::new()
        // World info
        .route("/api/world", get(get_world))
        // Entities
        .route("/api/entities", get(list_entities))
        .route("/api/entities/{id}", get(get_entity))
        // Players (convenience endpoint)
        .route("/api/players", get(list_players))
        // Chunks
        .route("/api/chunks", get(list_chunks))
        // History
        .route("/api/history/entity/{id}", get(get_entity_history))
        .with_state(state)
        .layer(cors)
}

/// Start the dashboard server on the given port.
pub async fn start_server(state: DashboardState, port: u16) {
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind dashboard server");

    tracing::info!("Dashboard server listening on http://localhost:{port}");

    axum::serve(listener, app)
        .await
        .expect("Dashboard server failed");
}

// ============================================================================
// Handlers
// ============================================================================

async fn get_world(State(state): State<DashboardState>) -> impl IntoResponse {
    let (tx, rx) = bounded(1);
    let request = DashboardRequest::GetWorld { response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Game loop not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(info) => Json(info).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct ListParams {
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn list_entities(
    State(state): State<DashboardState>,
    axum::extract::Query(params): axum::extract::Query<ListParams>,
) -> impl IntoResponse {
    let (tx, rx) = bounded(1);
    let request = DashboardRequest::ListEntities {
        limit: params.limit.unwrap_or(100),
        offset: params.offset.unwrap_or(0),
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Game loop not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(entities) => Json(entities).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn get_entity(State(state): State<DashboardState>, Path(id): Path<u64>) -> impl IntoResponse {
    let (tx, rx) = bounded(1);
    let request = DashboardRequest::GetEntity { id, response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Game loop not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(Some(entity)) => Json(entity).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Entity not found"})),
        )
            .into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn list_players(State(state): State<DashboardState>) -> impl IntoResponse {
    let (tx, rx) = bounded(1);
    let request = DashboardRequest::ListPlayers { response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Game loop not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(players) => Json(players).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn list_chunks(State(state): State<DashboardState>) -> impl IntoResponse {
    let (tx, rx) = bounded(1);
    let request = DashboardRequest::ListChunks { response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Game loop not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(chunks) => Json(chunks).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn get_entity_history(
    State(state): State<DashboardState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let (tx, rx) = bounded(1);
    let request = DashboardRequest::GetEntityHistory { id, response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Game loop not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(history) => Json(history).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}
