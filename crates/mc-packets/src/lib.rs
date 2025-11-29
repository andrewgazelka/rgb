// Auto-generated Minecraft packet definitions
// Run `nix run .#mc-gen` to regenerate

/// Protocol version for this build
pub const PROTOCOL_VERSION: i32 = 1073742105;

/// Minecraft version name for this build
pub const PROTOCOL_NAME: &str = "1.21.11-pre3";

// Re-export protocol types
pub use mc_protocol::{State, Direction, Packet};

pub mod handshake;
pub mod status;
pub mod login;
pub mod configuration;
pub mod play;
