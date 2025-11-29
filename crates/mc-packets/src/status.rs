// Auto-generated from Minecraft - status
// Do not edit manually

#![allow(dead_code)]

use std::borrow::Cow;
use mc_protocol::{Encode, Decode, VarInt, Uuid, Position, Nbt, BlockState};
use serde::{Serialize, Deserialize};

pub mod clientbound {
    use super::*;

    /// StatusResponse (ID: 0)
    pub const STATUS_RESPONSE_ID: i32 = 0;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StatusResponse;

    /// PongResponse (ID: 1)
    pub const PONG_RESPONSE_ID: i32 = 1;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PongResponse;

}

pub mod serverbound {
    use super::*;

    /// StatusRequest (ID: 0)
    pub const STATUS_REQUEST_ID: i32 = 0;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StatusRequest;

    /// PingRequest (ID: 1)
    pub const PING_REQUEST_ID: i32 = 1;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PingRequest;

}
