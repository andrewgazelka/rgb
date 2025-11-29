package dev.rgb.testagent

import net.fabricmc.api.ClientModInitializer
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents
import org.slf4j.LoggerFactory

object RgbTestAgentMod : ClientModInitializer {
    private val logger = LoggerFactory.getLogger("rgb-test-agent")

    private var testServer: TestAgentServer? = null

    override fun onInitializeClient() {
        val socketPath = System.getProperty("rgb.test.socket")
            ?: System.getenv("RGB_TEST_SOCKET")

        if (socketPath == null) {
            logger.info("RGB Test Agent: No socket path configured, running in normal mode")
            return
        }

        logger.info("RGB Test Agent: Starting with socket at $socketPath")

        testServer = TestAgentServer(socketPath).also { server ->
            server.start()

            ClientTickEvents.END_CLIENT_TICK.register { client ->
                server.processTick(client)
            }
        }

        logger.info("RGB Test Agent: Initialized successfully")
    }

    fun getServer(): TestAgentServer? = testServer
}
