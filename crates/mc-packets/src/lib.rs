/// Protocol version for this build
pub const PROTOCOL_VERSION: i32 = 1073742105i32;
/// Minecraft version name for this build
pub const PROTOCOL_NAME: &str = "1.21.11-pre3";
pub use mc_protocol::{Direction, Packet, State};
pub mod configuration;
pub mod handshake;
pub mod login;
pub mod play;
pub mod status;
