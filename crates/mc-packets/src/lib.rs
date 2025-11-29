//! Auto-generated Minecraft packet definitions
//!
//! Run `nix run .#mc-gen` to regenerate the JSON data files.

#![allow(dead_code)]
#![allow(unused_imports)]

// Re-export protocol types
pub use mc_protocol::{Direction, Packet, State};

// Include generated constants
include!(concat!(env!("OUT_DIR"), "/constants.rs"));

/// Handshake state packets
pub mod handshake {
    include!(concat!(env!("OUT_DIR"), "/handshake.rs"));
}

/// Status state packets
pub mod status {
    include!(concat!(env!("OUT_DIR"), "/status.rs"));
}

/// Login state packets
pub mod login {
    include!(concat!(env!("OUT_DIR"), "/login.rs"));
}

/// Configuration state packets
pub mod configuration {
    include!(concat!(env!("OUT_DIR"), "/configuration.rs"));
}

/// Play state packets
pub mod play {
    include!(concat!(env!("OUT_DIR"), "/play.rs"));
}

// Include generated block registry
mod block_registry {
    include!(concat!(env!("OUT_DIR"), "/blocks.rs"));
}

// Re-export block types at crate root
pub use block_registry::BlockState;
pub use block_registry::blocks;
