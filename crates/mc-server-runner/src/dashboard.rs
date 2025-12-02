//! ECS Dashboard web server for introspection.
//!
//! Provides REST API endpoints for inspecting and modifying ECS state
//! from a web dashboard.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use crossbeam_channel::Sender;
use rgb_ecs_introspect::{
    ChangeSource, HistoryStore, IntrospectChannels, IntrospectRegistry, IntrospectRequest,
    QuerySpec, protocol::oneshot,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

/// Shared state for the dashboard API.
#[derive(Clone)]
pub struct DashboardState {
    /// Channel to send requests to the ECS world.
    request_tx: Sender<IntrospectRequest>,
    /// Registry of introspectable components.
    registry: Arc<IntrospectRegistry>,
    /// Component change history.
    history: HistoryStore,
}

impl DashboardState {
    /// Create dashboard state from introspect channels and registry.
    pub fn new(channels: &IntrospectChannels, registry: Arc<IntrospectRegistry>) -> Self {
        Self {
            request_tx: channels.request_tx.clone(),
            registry,
            history: HistoryStore::default(),
        }
    }
}

/// Request timeout for ECS operations.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Create the dashboard router with all endpoints.
pub fn create_router(state: DashboardState) -> Router {
    // Permissive CORS for development - allows any origin
    let cors = CorsLayer::permissive();

    let api = Router::new()
        // World endpoints
        .route("/api/world", get(get_world))
        // Entity endpoints
        .route("/api/entities", get(list_entities).post(spawn_entity))
        .route("/api/entities/{id}", get(get_entity).delete(despawn_entity))
        // Component endpoints
        .route(
            "/api/entities/{id}/components/{name}",
            get(get_component)
                .put(update_component)
                .post(add_component)
                .delete(remove_component),
        )
        // Query endpoint
        .route("/api/query", post(execute_query))
        // Component types endpoint
        .route("/api/component-types", get(get_component_types))
        // Chunks endpoint (for map view)
        .route("/api/chunks", get(get_chunks))
        // History endpoints
        .route("/api/history", get(get_global_history))
        .route("/api/history/entity/{id}", get(get_entity_history))
        .route(
            "/api/history/entity/{id}/component/{name}",
            get(get_component_history),
        )
        .route("/api/history/revert/{entry_id}", post(revert_to_entry))
        .with_state(state);

    // CORS layer must be outermost to handle OPTIONS preflight
    api.layer(cors)
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
    let (tx, rx) = oneshot::channel();
    let request = IntrospectRequest::GetWorld { response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => Json(response).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct ListEntitiesParams {
    filter: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn list_entities(
    State(state): State<DashboardState>,
    axum::extract::Query(params): axum::extract::Query<ListEntitiesParams>,
) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let filter = params
        .filter
        .map(|f| f.split(',').map(String::from).collect());
    let request = IntrospectRequest::ListEntities {
        filter,
        limit: params.limit,
        offset: params.offset,
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => Json(response).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn get_entity(State(state): State<DashboardState>, Path(id): Path<u64>) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let entity = rgb_ecs::Entity::from_bits(id);
    let request = IntrospectRequest::GetEntity {
        entity,
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => {
            if response.found {
                Json(response).into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Entity not found"})),
                )
                    .into_response()
            }
        }
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn get_component(
    State(state): State<DashboardState>,
    Path((id, name)): Path<(u64, String)>,
) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let entity = rgb_ecs::Entity::from_bits(id);
    let request = IntrospectRequest::GetComponent {
        entity,
        component: name,
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => {
            if response.found {
                Json(response).into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Component not found"})),
                )
                    .into_response()
            }
        }
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn update_component(
    State(state): State<DashboardState>,
    Path((id, name)): Path<(u64, String)>,
    Json(value): Json<serde_json::Value>,
) -> impl IntoResponse {
    let entity = rgb_ecs::Entity::from_bits(id);

    // First, get the old value for history
    let old_value = {
        let (tx, rx) = oneshot::channel();
        let request = IntrospectRequest::GetComponent {
            entity,
            component: name.clone(),
            response: tx,
        };
        if state.request_tx.send(request).is_ok() {
            rx.recv_timeout(REQUEST_TIMEOUT)
                .ok()
                .and_then(|r| r.value)
                .map(|v| v.value)
        } else {
            None
        }
    };

    // Now perform the update
    let (tx, rx) = oneshot::channel();
    let request = IntrospectRequest::UpdateComponent {
        entity,
        component: name.clone(),
        value: value.clone(),
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => {
            if response.success {
                // Record in history
                state
                    .history
                    .record(id, name, old_value, Some(value), ChangeSource::Dashboard);
                Json(response).into_response()
            } else {
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
        }
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn add_component(
    State(state): State<DashboardState>,
    Path((id, name)): Path<(u64, String)>,
    Json(value): Json<serde_json::Value>,
) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let entity = rgb_ecs::Entity::from_bits(id);
    let request = IntrospectRequest::AddComponent {
        entity,
        component: name,
        value,
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => {
            if response.success {
                (StatusCode::CREATED, Json(response)).into_response()
            } else {
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
        }
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn remove_component(
    State(state): State<DashboardState>,
    Path((id, name)): Path<(u64, String)>,
) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let entity = rgb_ecs::Entity::from_bits(id);
    let request = IntrospectRequest::RemoveComponent {
        entity,
        component: name,
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => {
            if response.success {
                Json(response).into_response()
            } else {
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
        }
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct SpawnEntityRequest {
    name: Option<String>,
    components: Vec<(String, serde_json::Value)>,
}

async fn spawn_entity(
    State(state): State<DashboardState>,
    Json(body): Json<SpawnEntityRequest>,
) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let request = IntrospectRequest::SpawnEntity {
        name: body.name,
        components: body.components,
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => {
            if response.success {
                (StatusCode::CREATED, Json(response)).into_response()
            } else {
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
        }
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn despawn_entity(
    State(state): State<DashboardState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let entity = rgb_ecs::Entity::from_bits(id);
    let request = IntrospectRequest::DespawnEntity {
        entity,
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => {
            if response.success {
                Json(response).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json(response)).into_response()
            }
        }
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn execute_query(
    State(state): State<DashboardState>,
    Json(spec): Json<QuerySpec>,
) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let request = IntrospectRequest::Query { spec, response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => Json(response).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn get_component_types(State(state): State<DashboardState>) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let request = IntrospectRequest::GetComponentTypes { response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => Json(response).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

async fn get_chunks(State(state): State<DashboardState>) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let request = IntrospectRequest::GetChunks { response: tx };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => Json(response).into_response(),
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"error": "Request timeout"})),
        )
            .into_response(),
    }
}

// ============================================================================
// History Handlers
// ============================================================================

#[derive(Deserialize)]
struct HistoryQuery {
    limit: Option<usize>,
}

async fn get_global_history(
    State(state): State<DashboardState>,
    axum::extract::Query(params): axum::extract::Query<HistoryQuery>,
) -> impl IntoResponse {
    let entries = state.history.get_global_history(params.limit);
    Json(serde_json::json!({
        "entries": entries,
        "total": entries.len()
    }))
}

async fn get_entity_history(
    State(state): State<DashboardState>,
    Path(id): Path<u64>,
    axum::extract::Query(params): axum::extract::Query<HistoryQuery>,
) -> impl IntoResponse {
    let entries = state.history.get_entity_history(id, params.limit);
    Json(serde_json::json!({
        "entries": entries,
        "total": entries.len()
    }))
}

async fn get_component_history(
    State(state): State<DashboardState>,
    Path((id, name)): Path<(u64, String)>,
    axum::extract::Query(params): axum::extract::Query<HistoryQuery>,
) -> impl IntoResponse {
    let entries = state.history.get_component_history(id, &name, params.limit);
    Json(serde_json::json!({
        "entries": entries,
        "total": entries.len()
    }))
}

async fn revert_to_entry(
    State(state): State<DashboardState>,
    Path(entry_id): Path<u64>,
) -> impl IntoResponse {
    // Get the history entry
    let entry = match state.history.get_entry(entry_id) {
        Some(e) => e,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"success": false, "error": "History entry not found"})),
            )
                .into_response();
        }
    };

    // Get the value to revert to (the old_value from this entry)
    let revert_value = match &entry.old_value {
        Some(v) => v.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Cannot revert: no previous value (component was added)"
                })),
            )
                .into_response();
        }
    };

    // Send update request
    let (tx, rx) = oneshot::channel();
    let entity = rgb_ecs::Entity::from_bits(entry.entity);
    let request = IntrospectRequest::UpdateComponent {
        entity,
        component: entry.component.clone(),
        value: revert_value.clone(),
        response: tx,
    };

    if state.request_tx.send(request).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"success": false, "error": "ECS not available"})),
        )
            .into_response();
    }

    match rx.recv_timeout(REQUEST_TIMEOUT) {
        Ok(response) => {
            if response.success {
                // Record the revert in history
                state.history.record(
                    entry.entity,
                    entry.component.clone(),
                    entry.new_value.clone(),
                    Some(revert_value),
                    ChangeSource::Revert,
                );
                Json(serde_json::json!({
                    "success": true,
                    "reverted_to": entry_id
                }))
                .into_response()
            } else {
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
        }
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"success": false, "error": "Request timeout"})),
        )
            .into_response(),
    }
}
