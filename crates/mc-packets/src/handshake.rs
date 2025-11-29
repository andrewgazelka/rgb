// Auto-generated from Minecraft - handshake
// Do not edit manually

#![allow(dead_code)]

use std::borrow::Cow;
use mc_protocol::{Encode, Decode, Packet, State, Direction, VarInt, Uuid, Position, Nbt, BlockState};
use serde::{Serialize, Deserialize};

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
