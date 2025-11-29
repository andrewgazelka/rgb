(change this document as you progress)

## Architecture

We now have:
- `mc-client` - Rust client that connects to servers and records packets
- `mc-server` - Rust Minecraft server implementation
- `mc-proxy` - Proxy that sits between client and server, records all packets
- `mc-packets` - Auto-generated packet definitions with serde support
- `mc-protocol` - Protocol encoding/decoding traits

The Java server can be run via: `nix run .#run-mc-server`

## Completed Tasks

- [x] Add Serialize/Deserialize to packet structs in mc-packets
  - Added serde dependency to mc-protocol and mc-packets
  - Updated all protocol types (VarInt, VarLong, Uuid, Position, Nbt, BlockState) with Serialize/Deserialize
  - Updated gen_packets.py to generate serde derives on all packet structs
  - Regenerated packets with `nix run .#mc-gen`

- [x] Create mc-client for connecting to Java server and recording packets
  - Handles handshake, login, configuration, and play states
  - Records all packets with timestamps to JSON
  - Usage: `cargo run -p mc-client -- <host> <port> <player_name> <output.json>`

- [x] Update mc-proxy to record packets for replay
  - Saves all packets with timestamps, state, direction, and raw data
  - Outputs to JSON for analysis
  - Usage: `cargo run -p mc-proxy -- <listen_port> <target_port> <output.json>`

## Remaining Tasks

- [ ] Use client to join native Java server and record packet sequence
  - Run Java server: `nix run .#run-mc-server -- 25566`
  - Connect with Rust client: `cargo run -p mc-client -- 127.0.0.1 25566 TestBot java-packets.json`

- [ ] Make sure the actual Rust server implementation is 1:1 with Java
  - Compare recorded packets from Java server with Rust server output
  - Ensure same packet sequence, IDs, and data formats
