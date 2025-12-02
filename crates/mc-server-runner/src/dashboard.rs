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
    routing::{get, post},
};
use crossbeam_channel::{Receiver, Sender, bounded};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::persistence::{ChangeSource, PersistenceDb};

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
    Query {
        spec: QuerySpec,
        response: Sender<QueryResponse>,
    },
}

/// Query specification for filtering entities.
#[derive(Deserialize, Clone, Default)]
pub struct QuerySpec {
    #[serde(default)]
    pub with: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
    #[serde(default)]
    pub without: Vec<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

/// Query result row.
#[derive(Serialize, Clone)]
pub struct QueryResultRow {
    pub entity: u64,
    pub name: Option<String>,
    pub components: std::collections::HashMap<String, serde_json::Value>,
}

/// Query response.
#[derive(Serialize, Clone)]
pub struct QueryResponse {
    pub entities: Vec<QueryResultRow>,
    pub total: usize,
    pub execution_time_us: u64,
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
    pub found: bool,
    pub id: u64,
    pub name: Option<String>,
    pub components: Vec<ComponentValue>,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
}

#[derive(Serialize, Clone)]
pub struct ComponentValue {
    pub name: String,
    pub full_name: String,
    pub value: serde_json::Value,
    pub is_opaque: bool,
    pub schema: Option<serde_json::Value>,
}

#[derive(Serialize, Clone)]
pub struct ChunkInfo {
    pub x: i32,
    pub z: i32,
    pub color: String,
    pub loaded: bool,
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
    /// Persistence database for history.
    persistence_db: Option<PersistenceDb>,
}

impl DashboardState {
    /// Create dashboard state from channels.
    pub fn new(channels: &DashboardChannels) -> Self {
        Self {
            request_tx: channels.request_tx.clone(),
            persistence_db: None,
        }
    }

    /// Create dashboard state with persistence database.
    pub fn with_persistence(channels: &DashboardChannels, db: PersistenceDb) -> Self {
        Self {
            request_tx: channels.request_tx.clone(),
            persistence_db: Some(db),
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
        // Query
        .route("/api/query", post(query_entities))
        // History endpoints
        .route("/api/history", get(get_global_history))
        .route("/api/history/entity/{id}", get(get_entity_history))
        .route(
            "/api/history/entity/{entity_id}/component/{component}",
            get(get_component_history),
        )
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

async fn query_entities(
    State(state): State<DashboardState>,
    Json(spec): Json<QuerySpec>,
) -> impl IntoResponse {
    let (tx, rx) = bounded(1);
    let request = DashboardRequest::Query { spec, response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Game loop not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(result) => Json(result).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

// ============================================================================
// History Response Types
// ============================================================================

/// History entry for API response.
#[derive(Serialize, Clone)]
pub struct HistoryEntryResponse {
    pub id: u64,
    pub timestamp: u64,
    pub entity: u64,
    pub component: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub source: ChangeSource,
}

/// History response.
#[derive(Serialize)]
pub struct HistoryResponse {
    pub entries: Vec<HistoryEntryResponse>,
    pub total: usize,
}

// ============================================================================
// History Handlers
// ============================================================================

#[derive(Deserialize)]
struct HistoryParams {
    limit: Option<usize>,
}

async fn get_global_history(
    State(state): State<DashboardState>,
    axum::extract::Query(params): axum::extract::Query<HistoryParams>,
) -> impl IntoResponse {
    let Some(db) = &state.persistence_db else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Persistence not enabled"})),
        )
            .into_response();
    };

    let limit = params.limit.unwrap_or(50);
    match db.get_global_history(limit) {
        Ok(entries) => {
            let response = HistoryResponse {
                total: entries.len(),
                entries: entries
                    .into_iter()
                    .map(|e| HistoryEntryResponse {
                        id: e.id,
                        timestamp: e.timestamp,
                        entity: e.entity_id,
                        component: e.component,
                        old_value: e.old_value,
                        new_value: e.new_value,
                        source: e.source,
                    })
                    .collect(),
            };
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to get history: {e}")})),
        )
            .into_response(),
    }
}

async fn get_entity_history(
    State(state): State<DashboardState>,
    Path(id): Path<u64>,
    axum::extract::Query(params): axum::extract::Query<HistoryParams>,
) -> impl IntoResponse {
    let Some(db) = &state.persistence_db else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Persistence not enabled"})),
        )
            .into_response();
    };

    // The ID is the Flecs entity ID. We need to look up the UUID.
    // For now, we'll use the entity ID as a workaround since we store both.
    // In a real implementation, we'd query the game loop for the UUID.

    // HACK: Use the entity_id to find entries (not ideal, but works for now)
    let limit = params.limit.unwrap_or(50);
    match db.get_global_history(limit * 10) {
        Ok(entries) => {
            let filtered: Vec<_> = entries
                .into_iter()
                .filter(|e| e.entity_id == id)
                .take(limit)
                .map(|e| HistoryEntryResponse {
                    id: e.id,
                    timestamp: e.timestamp,
                    entity: e.entity_id,
                    component: e.component,
                    old_value: e.old_value,
                    new_value: e.new_value,
                    source: e.source,
                })
                .collect();

            let response = HistoryResponse {
                total: filtered.len(),
                entries: filtered,
            };
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to get history: {e}")})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct ComponentHistoryPath {
    entity_id: u64,
    component: String,
}

async fn get_component_history(
    State(state): State<DashboardState>,
    Path(path): Path<ComponentHistoryPath>,
    axum::extract::Query(params): axum::extract::Query<HistoryParams>,
) -> impl IntoResponse {
    let Some(db) = &state.persistence_db else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Persistence not enabled"})),
        )
            .into_response();
    };

    let limit = params.limit.unwrap_or(50);
    match db.get_global_history(limit * 10) {
        Ok(entries) => {
            let filtered: Vec<_> = entries
                .into_iter()
                .filter(|e| e.entity_id == path.entity_id && e.component == path.component)
                .take(limit)
                .map(|e| HistoryEntryResponse {
                    id: e.id,
                    timestamp: e.timestamp,
                    entity: e.entity_id,
                    component: e.component,
                    old_value: e.old_value,
                    new_value: e.new_value,
                    source: e.source,
                })
                .collect();

            let response = HistoryResponse {
                total: filtered.len(),
                entries: filtered,
            };
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to get history: {e}")})),
        )
            .into_response(),
    }
}
