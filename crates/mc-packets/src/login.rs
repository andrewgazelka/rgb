// Auto-generated from Minecraft - login
// Do not edit manually

#![allow(dead_code)]
#![allow(unused_imports)]

use mc_protocol::{
    BlockState, Decode, Direction, Encode, Nbt, Packet, Position, State, Uuid, VarInt,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub mod clientbound {
    use super::*;

    /// Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LoginDisconnect;

    impl Packet for LoginDisconnect {
        const ID: i32 = 0;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Hello;

    impl Packet for Hello {
        const ID: i32 = 1;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 2
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LoginFinished;

    impl Packet for LoginFinished {
        const ID: i32 = 2;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 3
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LoginCompression;

    impl Packet for LoginCompression {
        const ID: i32 = 3;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 4
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomQuery;

    impl Packet for CustomQuery {
        const ID: i32 = 4;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 5
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieRequest;

    impl Packet for CookieRequest {
        const ID: i32 = 5;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Clientbound;
    }
}

pub mod serverbound {
    use super::*;

    /// Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Hello;

    impl Packet for Hello {
        const ID: i32 = 0;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Key;

    impl Packet for Key {
        const ID: i32 = 1;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 2
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomQueryAnswer;

    impl Packet for CustomQueryAnswer {
        const ID: i32 = 2;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 3
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LoginAcknowledged;

    impl Packet for LoginAcknowledged {
        const ID: i32 = 3;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 4
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieResponse;

    impl Packet for CookieResponse {
        const ID: i32 = 4;
        const STATE: State = State::Login;
        const DIRECTION: Direction = Direction::Serverbound;
    }
}
