//! JSON-RPC 2.0 protocol types for communicating with the Fabric test agent.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: Option<Value>, id: i64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: Some(id),
        }
    }
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<i64>,
}

impl JsonRpcResponse {
    /// Check if this is an error response
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get the result, or an error if this is an error response
    ///
    /// # Errors
    /// Returns an error if the response contains an error field
    pub fn into_result(self) -> eyre::Result<Value> {
        if let Some(error) = self.error {
            eyre::bail!("JSON-RPC error {}: {}", error.code, error.message);
        }
        Ok(self.result.unwrap_or(Value::Null))
    }
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// Error codes
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;
pub const CONNECTION_ERROR: i32 = -32000;
pub const TIMEOUT: i32 = -32001;
pub const NOT_CONNECTED: i32 = -32002;

// Command parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectParams {
    pub host: String,
    pub port: u16,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitForStateParams {
    pub state: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitForChunksParams {
    pub count: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveToParams {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub connected: bool,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Rotation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_mode: Option<String>,
    #[serde(default)]
    pub chunk_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

// Event types (received as notifications)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TestEvent {
    Connected {
        host: String,
        port: u16,
    },
    LoginSuccess {
        uuid: String,
        username: String,
    },
    PlayState {
        entity_id: i32,
    },
    ChunkLoaded {
        x: i32,
        z: i32,
    },
    PositionSync {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
    },
    Disconnected {
        reason: String,
    },
}

/// Parse an event notification from a JSON-RPC request
pub fn parse_event(request: &JsonRpcRequest) -> Option<TestEvent> {
    if request.method != "event" {
        return None;
    }

    let params = request.params.as_ref()?;
    let data = params.get("data")?;
    serde_json::from_value(data.clone()).ok()
}
