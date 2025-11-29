// Auto-generated from Minecraft - status
// Do not edit manually

#![allow(dead_code)]

use std::borrow::Cow;
use mc_protocol::{Encode, Decode, Packet, State, Direction, VarInt, Uuid, Position, Nbt, BlockState};
use serde::{Serialize, Deserialize};

pub mod clientbound {
    use super::*;

    /// Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StatusResponse;

    impl Packet for StatusResponse {
        const ID: i32 = 0;
        const STATE: State = State::Status;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PongResponse;

    impl Packet for PongResponse {
        const ID: i32 = 1;
        const STATE: State = State::Status;
        const DIRECTION: Direction = Direction::Clientbound;
    }

}

pub mod serverbound {
    use super::*;

    /// Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StatusRequest;

    impl Packet for StatusRequest {
        const ID: i32 = 0;
        const STATE: State = State::Status;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PingRequest;

    impl Packet for PingRequest {
        const ID: i32 = 1;
        const STATE: State = State::Status;
        const DIRECTION: Direction = Direction::Serverbound;
    }

}
