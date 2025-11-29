#![allow(dead_code)]
#![allow(unused_imports)]
use mc_protocol::{
    BlockState, Decode, Direction, Encode, Nbt, Packet, Position, State, Uuid, VarInt,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
pub mod clientbound {
    use super::*;
    ///Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StatusResponse;
    impl Packet for StatusResponse {
        const ID: i32 = 0i32;
        const STATE: State = State::Status;
        const DIRECTION: Direction = Direction::Clientbound;
    }
    ///Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PongResponse;
    impl Packet for PongResponse {
        const ID: i32 = 1i32;
        const STATE: State = State::Status;
        const DIRECTION: Direction = Direction::Clientbound;
    }
}
pub mod serverbound {
    use super::*;
    ///Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StatusRequest;
    impl Packet for StatusRequest {
        const ID: i32 = 0i32;
        const STATE: State = State::Status;
        const DIRECTION: Direction = Direction::Serverbound;
    }
    ///Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PingRequest;
    impl Packet for PingRequest {
        const ID: i32 = 1i32;
        const STATE: State = State::Status;
        const DIRECTION: Direction = Direction::Serverbound;
    }
}
