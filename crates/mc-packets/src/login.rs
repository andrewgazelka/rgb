// Auto-generated from Minecraft - login
// Do not edit manually

#![allow(dead_code)]

use std::borrow::Cow;
use mc_protocol::{Encode, Decode, VarInt, Uuid, Position, Nbt, BlockState};
use serde::{Serialize, Deserialize};

pub mod clientbound {
    use super::*;

    /// LoginDisconnect (ID: 0)
    pub const LOGIN_DISCONNECT_ID: i32 = 0;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LoginDisconnect;

    /// Hello (ID: 1)
    pub const HELLO_ID: i32 = 1;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Hello;

    /// LoginFinished (ID: 2)
    pub const LOGIN_FINISHED_ID: i32 = 2;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LoginFinished;

    /// LoginCompression (ID: 3)
    pub const LOGIN_COMPRESSION_ID: i32 = 3;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LoginCompression;

    /// CustomQuery (ID: 4)
    pub const CUSTOM_QUERY_ID: i32 = 4;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomQuery;

    /// CookieRequest (ID: 5)
    pub const COOKIE_REQUEST_ID: i32 = 5;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieRequest;

}

pub mod serverbound {
    use super::*;

    /// Hello (ID: 0)
    pub const HELLO_ID: i32 = 0;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Hello;

    /// Key (ID: 1)
    pub const KEY_ID: i32 = 1;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Key;

    /// CustomQueryAnswer (ID: 2)
    pub const CUSTOM_QUERY_ANSWER_ID: i32 = 2;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomQueryAnswer;

    /// LoginAcknowledged (ID: 3)
    pub const LOGIN_ACKNOWLEDGED_ID: i32 = 3;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LoginAcknowledged;

    /// CookieResponse (ID: 4)
    pub const COOKIE_RESPONSE_ID: i32 = 4;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieResponse;

}
