package dev.rgb.testagent.commands

import dev.rgb.testagent.TestAgentServer
import dev.rgb.testagent.protocol.ConnectParams
import dev.rgb.testagent.protocol.JsonRpc
import dev.rgb.testagent.protocol.JsonRpcRequest
import dev.rgb.testagent.protocol.JsonRpcResponse
import dev.rgb.testagent.protocol.MoveToParams
import dev.rgb.testagent.protocol.PlayerState
import dev.rgb.testagent.protocol.Position
import dev.rgb.testagent.protocol.Rotation
import dev.rgb.testagent.protocol.json
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.encodeToJsonElement
import net.minecraft.client.MinecraftClient
import net.minecraft.client.gui.screen.multiplayer.ConnectScreen
import net.minecraft.client.network.ServerAddress
import net.minecraft.client.network.ServerInfo
import net.minecraft.network.packet.c2s.play.PlayerMoveC2SPacket
import net.minecraft.text.Text
import org.slf4j.LoggerFactory

class CommandHandler(private val server: TestAgentServer) {
    private val logger = LoggerFactory.getLogger("rgb-test-agent")

    // Track loaded chunks for reporting
    private val loadedChunks = mutableSetOf<Pair<Int, Int>>()

    fun handle(client: MinecraftClient, request: JsonRpcRequest): JsonRpcResponse {
        return when (request.method) {
            "connect" -> handleConnect(client, request)
            "disconnect" -> handleDisconnect(client, request)
            "get_player_state" -> handleGetPlayerState(client, request)
            "move_to" -> handleMoveTo(client, request)
            "get_loaded_chunks" -> handleGetLoadedChunks(request)
            "ping" -> JsonRpc.success(request.id, JsonPrimitive("pong"))
            else -> JsonRpc.error(request.id, JsonRpc.METHOD_NOT_FOUND, "Unknown method: ${request.method}")
        }
    }

    private fun handleConnect(client: MinecraftClient, request: JsonRpcRequest): JsonRpcResponse {
        val params = try {
            request.params?.let { json.decodeFromJsonElement(ConnectParams.serializer(), it) }
                ?: return JsonRpc.error(request.id, JsonRpc.INVALID_PARAMS, "Missing params")
        } catch (e: Exception) {
            return JsonRpc.error(request.id, JsonRpc.INVALID_PARAMS, "Invalid params: ${e.message}")
        }

        logger.info("Connecting to ${params.host}:${params.port} as ${params.username}")

        // Connect to server
        val serverAddress = ServerAddress(params.host, params.port)
        val serverInfo = ServerInfo("Test Server", serverAddress.address, ServerInfo.ServerType.OTHER)

        client.execute {
            ConnectScreen.connect(
                client.currentScreen,
                client,
                serverAddress,
                serverInfo,
                false,
                null
            )
        }

        return JsonRpc.successEmpty(request.id)
    }

    private fun handleDisconnect(client: MinecraftClient, request: JsonRpcRequest): JsonRpcResponse {
        client.execute {
            // Disconnect from server
            client.disconnect(Text.literal("Test disconnected"))
        }

        loadedChunks.clear()

        return JsonRpc.successEmpty(request.id)
    }

    private fun handleGetPlayerState(client: MinecraftClient, request: JsonRpcRequest): JsonRpcResponse {
        val player = client.player
        val networkHandler = client.networkHandler

        val state = PlayerState(
            connected = networkHandler != null,
            state = when {
                client.world != null && player != null -> "play"
                networkHandler != null -> "connecting"
                else -> "disconnected"
            },
            position = player?.let { Position(it.x, it.y, it.z) },
            rotation = player?.let { Rotation(it.yaw, it.pitch) },
            entityId = player?.id,
            gameMode = client.interactionManager?.currentGameMode?.name,
            chunkCount = loadedChunks.size
        )

        return JsonRpc.success(request.id, json.encodeToJsonElement(state))
    }

    private fun handleMoveTo(client: MinecraftClient, request: JsonRpcRequest): JsonRpcResponse {
        val params = try {
            request.params?.let { json.decodeFromJsonElement(MoveToParams.serializer(), it) }
                ?: return JsonRpc.error(request.id, JsonRpc.INVALID_PARAMS, "Missing params")
        } catch (e: Exception) {
            return JsonRpc.error(request.id, JsonRpc.INVALID_PARAMS, "Invalid params: ${e.message}")
        }

        val player = client.player
            ?: return JsonRpc.error(request.id, JsonRpc.NOT_CONNECTED, "Not in game")

        // Set player position and send to server
        player.setPosition(params.x, params.y, params.z)

        client.networkHandler?.sendPacket(
            PlayerMoveC2SPacket.PositionAndOnGround(
                params.x, params.y, params.z,
                player.isOnGround,
                player.horizontalCollision
            )
        )

        return JsonRpc.successEmpty(request.id)
    }

    private fun handleGetLoadedChunks(request: JsonRpcRequest): JsonRpcResponse {
        val chunks = loadedChunks.map { (x, z) ->
            dev.rgb.testagent.protocol.ChunkPos(x, z)
        }
        return JsonRpc.success(request.id, json.encodeToJsonElement(chunks))
    }

    // Called from mixins
    fun onChunkLoaded(x: Int, z: Int) {
        loadedChunks.add(x to z)
    }

    fun onChunkUnloaded(x: Int, z: Int) {
        loadedChunks.remove(x to z)
    }

    fun clearChunks() {
        loadedChunks.clear()
    }
}
