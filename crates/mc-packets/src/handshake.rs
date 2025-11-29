// Auto-generated from Minecraft - handshake
// Do not edit manually

#![allow(dead_code)]
#![allow(unused_imports)]

use mc_protocol::{
    BlockState, Decode, Direction, Encode, Nbt, Packet, Position, State, Uuid, VarInt,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub mod serverbound {
    use super::*;

    /// Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Intention;

    impl Packet for Intention {
        const ID: i32 = 0;
        const STATE: State = State::Handshaking;
        const DIRECTION: Direction = Direction::Serverbound;
    }
}
