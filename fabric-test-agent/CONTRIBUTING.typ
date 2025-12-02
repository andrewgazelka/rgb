= RGB Fabric Test Agent

Integration testing infrastructure for the RGB Minecraft server using a real Fabric client.

== Architecture

```
Rust Test Harness ←── Unix Socket (JSON-RPC) ──→ Fabric Test Agent (Kotlin)
        │                                                │
        │ spawns                                         │ connects
        ↓                                                ↓
   mc-server (Rust)                              Minecraft Client (1.21.11-pre4)
```

== Building

```bash
# Build the Fabric mod
./gradlew build

# Output: build/libs/rgb-test-agent-0.1.0.jar
```

== Running Tests

```bash
# From workspace root
MC_INTEGRATION_TESTS=1 cargo nextest run -p mc-integration-tests

# Or with Xvfb (headless)
nix run .#run-integration-tests
```

== Project Structure

```
fabric-test-agent/
├── build.gradle.kts          # Fabric Loom + Kotlin config
├── gradle.properties         # MC 1.21.11-pre4, Fabric versions
└── src/main/
    ├── kotlin/dev/rgb/testagent/
    │   ├── RgbTestAgentMod.kt      # Entrypoint, starts socket server
    │   ├── TestAgentServer.kt      # Unix socket JSON-RPC server
    │   ├── protocol/Messages.kt    # Request/response types
    │   └── commands/CommandHandler.kt
    ├── java/.../mixin/             # Event hooks (login, chunks, position)
    └── resources/
        ├── fabric.mod.json
        └── rgb-test-agent.mixins.json
```

== JSON-RPC Protocol

Commands (Rust → Fabric):
- `connect` — Connect to server with host, port, username
- `disconnect` — Disconnect from server
- `get_player_state` — Get position, state, entity ID, chunk count
- `move_to` — Move player to coordinates
- `get_loaded_chunks` — List loaded chunk positions
- `ping` — Connection health check

Events (Fabric → Rust):
- `login_success` — Player logged in with UUID
- `play_state` — Entered play state with entity ID
- `chunk_loaded` — Chunk received at (x, z)
- `position_sync` — Server sent position update

== Configuration

The mod activates when launched with:
```bash
java -Drgb.test.socket=/tmp/rgb-test-xxx.sock ...
```

Without this property, the mod is dormant.

== Dependencies

- Fabric Loader 0.18.1
- Fabric API 0.139.3+1.21.11
- Fabric Language Kotlin 1.12.3
- junixsocket 2.10.1 (Unix domain sockets)
- kotlinx-serialization-json 1.7.3
