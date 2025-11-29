// Auto-generated from Minecraft - handshake
// Do not edit manually

#![allow(dead_code)]

use std::borrow::Cow;
use mc_protocol::{Encode, Decode, VarInt, Uuid, Position, Nbt, BlockState};
use serde::{Serialize, Deserialize};

pub mod serverbound {
    use super::*;

    /// Intention (ID: 0)
    pub const INTENTION_ID: i32 = 0;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Intention;

}
