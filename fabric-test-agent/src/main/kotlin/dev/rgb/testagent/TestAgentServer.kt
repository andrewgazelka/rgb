package dev.rgb.testagent

import dev.rgb.testagent.commands.CommandHandler
import dev.rgb.testagent.protocol.JsonRpc
import dev.rgb.testagent.protocol.JsonRpcRequest
import dev.rgb.testagent.protocol.JsonRpcResponse
import dev.rgb.testagent.protocol.TestEvent
import dev.rgb.testagent.protocol.json
import kotlinx.serialization.encodeToString
import net.minecraft.client.MinecraftClient
import org.newsclub.net.unix.AFUNIXServerSocket
import org.newsclub.net.unix.AFUNIXSocketAddress
import org.slf4j.LoggerFactory
import java.io.BufferedReader
import java.io.BufferedWriter
import java.io.File
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.net.Socket
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.atomic.AtomicBoolean

class TestAgentServer(private val socketPath: String) {
    private val logger = LoggerFactory.getLogger("rgb-test-agent")

    private val running = AtomicBoolean(false)
    private val commandQueue = ConcurrentLinkedQueue<PendingCommand>()

    private var serverSocket: AFUNIXServerSocket? = null
    private var clientSocket: Socket? = null
    private var reader: BufferedReader? = null
    private var writer: BufferedWriter? = null

    private var serverThread: Thread? = null
    private var readerThread: Thread? = null

    private val commandHandler = CommandHandler(this)

    data class PendingCommand(
        val request: JsonRpcRequest,
        val responseCallback: (JsonRpcResponse) -> Unit
    )

    fun start() {
        if (running.getAndSet(true)) {
            logger.warn("Server already running")
            return
        }

        serverThread = Thread({
            runServer()
        }, "RGB-TestAgent-Server").apply {
            isDaemon = true
            start()
        }
    }

    private fun runServer() {
        try {
            val socketFile = File(socketPath)
            socketFile.parentFile?.mkdirs()
            socketFile.delete() // Clean up any stale socket

            val address = AFUNIXSocketAddress.of(socketFile)
            serverSocket = AFUNIXServerSocket.newInstance()
            serverSocket?.bind(address)

            logger.info("Listening on Unix socket: $socketPath")

            while (running.get()) {
                try {
                    val socket = serverSocket?.accept() ?: break
                    handleClient(socket)
                } catch (e: Exception) {
                    if (running.get()) {
                        logger.error("Error accepting connection", e)
                    }
                }
            }
        } catch (e: Exception) {
            logger.error("Server error", e)
        } finally {
            cleanup()
        }
    }

    private fun handleClient(socket: Socket) {
        // Only allow one client at a time
        clientSocket?.close()

        clientSocket = socket
        reader = BufferedReader(InputStreamReader(socket.getInputStream()))
        writer = BufferedWriter(OutputStreamWriter(socket.getOutputStream()))

        logger.info("Client connected")

        readerThread = Thread({
            readMessages()
        }, "RGB-TestAgent-Reader").apply {
            isDaemon = true
            start()
        }
    }

    private fun readMessages() {
        try {
            val r = reader ?: return

            while (running.get() && !Thread.currentThread().isInterrupted) {
                val line = r.readLine() ?: break

                try {
                    val request = json.decodeFromString<JsonRpcRequest>(line)
                    logger.debug("Received: ${request.method}")

                    // Queue command for main thread processing
                    commandQueue.offer(PendingCommand(request) { response ->
                        sendResponse(response)
                    })
                } catch (e: Exception) {
                    logger.error("Error parsing request: $line", e)
                    sendResponse(JsonRpc.error(null, JsonRpc.PARSE_ERROR, "Parse error: ${e.message}"))
                }
            }
        } catch (e: Exception) {
            if (running.get()) {
                logger.error("Reader error", e)
            }
        }

        logger.info("Client disconnected")
    }

    fun processTick(client: MinecraftClient) {
        // Process all pending commands on main thread
        while (true) {
            val pending = commandQueue.poll() ?: break

            try {
                val response = commandHandler.handle(client, pending.request)
                pending.responseCallback(response)
            } catch (e: Exception) {
                logger.error("Error handling command: ${pending.request.method}", e)
                pending.responseCallback(
                    JsonRpc.error(pending.request.id, JsonRpc.INTERNAL_ERROR, "Internal error: ${e.message}")
                )
            }
        }
    }

    @Synchronized
    fun sendResponse(response: JsonRpcResponse) {
        try {
            val w = writer ?: return
            val line = json.encodeToString(response)
            w.write(line)
            w.newLine()
            w.flush()
        } catch (e: Exception) {
            logger.error("Error sending response", e)
        }
    }

    @Synchronized
    fun sendEvent(event: TestEvent) {
        try {
            val w = writer ?: return
            val line = JsonRpc.eventNotification(event)
            w.write(line)
            w.newLine()
            w.flush()
        } catch (e: Exception) {
            logger.error("Error sending event", e)
        }
    }

    fun stop() {
        running.set(false)
        cleanup()
    }

    private fun cleanup() {
        try {
            reader?.close()
            writer?.close()
            clientSocket?.close()
            serverSocket?.close()
            File(socketPath).delete()
        } catch (e: Exception) {
            logger.error("Error during cleanup", e)
        }
    }
}
