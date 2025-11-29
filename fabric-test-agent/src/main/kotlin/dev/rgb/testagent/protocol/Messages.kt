package dev.rgb.testagent.protocol

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.encodeToJsonElement
import kotlinx.serialization.json.put

val json = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}

// JSON-RPC 2.0 Request
@Serializable
data class JsonRpcRequest(
    val jsonrpc: String = "2.0",
    val method: String,
    val params: JsonElement? = null,
    val id: Int? = null
)

// JSON-RPC 2.0 Response
@Serializable
data class JsonRpcResponse(
    val jsonrpc: String = "2.0",
    val result: JsonElement? = null,
    val error: JsonRpcError? = null,
    val id: Int? = null
)

@Serializable
data class JsonRpcError(
    val code: Int,
    val message: String,
    val data: JsonElement? = null
)

// Command parameters
@Serializable
data class ConnectParams(
    val host: String,
    val port: Int,
    val username: String
)

@Serializable
data class WaitForStateParams(
    val state: String,
    @SerialName("timeout_ms") val timeoutMs: Long
)

@Serializable
data class WaitForChunksParams(
    val count: Int,
    @SerialName("timeout_ms") val timeoutMs: Long
)

@Serializable
data class MoveToParams(
    val x: Double,
    val y: Double,
    val z: Double
)

// Response data types
@Serializable
data class Position(
    val x: Double,
    val y: Double,
    val z: Double
)

@Serializable
data class Rotation(
    val yaw: Float,
    val pitch: Float
)

@Serializable
data class PlayerState(
    val connected: Boolean,
    val state: String,
    val position: Position? = null,
    val rotation: Rotation? = null,
    @SerialName("entity_id") val entityId: Int? = null,
    @SerialName("game_mode") val gameMode: String? = null,
    @SerialName("chunk_count") val chunkCount: Int = 0
)

@Serializable
data class ChunkPos(
    val x: Int,
    val z: Int
)

// Event types for notifications
@Serializable
sealed class TestEvent {
    abstract val type: String

    @Serializable
    @SerialName("connected")
    data class Connected(
        override val type: String = "connected",
        val host: String,
        val port: Int
    ) : TestEvent()

    @Serializable
    @SerialName("login_success")
    data class LoginSuccess(
        override val type: String = "login_success",
        val uuid: String,
        val username: String
    ) : TestEvent()

    @Serializable
    @SerialName("play_state")
    data class PlayState(
        override val type: String = "play_state",
        @SerialName("entity_id") val entityId: Int
    ) : TestEvent()

    @Serializable
    @SerialName("chunk_loaded")
    data class ChunkLoaded(
        override val type: String = "chunk_loaded",
        val x: Int,
        val z: Int
    ) : TestEvent()

    @Serializable
    @SerialName("position_sync")
    data class PositionSync(
        override val type: String = "position_sync",
        val x: Double,
        val y: Double,
        val z: Double,
        val yaw: Float,
        val pitch: Float
    ) : TestEvent()

    @Serializable
    @SerialName("disconnected")
    data class Disconnected(
        override val type: String = "disconnected",
        val reason: String
    ) : TestEvent()
}

// Helper functions
object JsonRpc {
    const val PARSE_ERROR = -32700
    const val INVALID_REQUEST = -32600
    const val METHOD_NOT_FOUND = -32601
    const val INVALID_PARAMS = -32602
    const val INTERNAL_ERROR = -32603
    const val CONNECTION_ERROR = -32000
    const val TIMEOUT = -32001
    const val NOT_CONNECTED = -32002

    fun success(id: Int?, result: JsonElement): JsonRpcResponse =
        JsonRpcResponse(result = result, id = id)

    fun successEmpty(id: Int?): JsonRpcResponse =
        JsonRpcResponse(result = JsonObject(emptyMap()), id = id)

    fun error(id: Int?, code: Int, message: String, data: JsonElement? = null): JsonRpcResponse =
        JsonRpcResponse(error = JsonRpcError(code, message, data), id = id)

    fun notification(method: String, params: JsonElement): JsonRpcRequest =
        JsonRpcRequest(method = method, params = params, id = null)

    fun eventNotification(event: TestEvent): String {
        val params = buildJsonObject {
            put("type", event.type)
            put("data", json.encodeToJsonElement(event))
        }
        return json.encodeToString(JsonRpcRequest.serializer(), notification("event", params))
    }
}
