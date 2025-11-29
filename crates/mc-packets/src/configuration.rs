// Auto-generated from Minecraft - configuration
// Do not edit manually

#![allow(dead_code)]

use std::borrow::Cow;
use mc_protocol::{Encode, Decode, Packet, State, Direction, VarInt, Uuid, Position, Nbt, BlockState};
use serde::{Serialize, Deserialize};

pub mod clientbound {
    use super::*;

    /// Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieRequest;

    impl Packet for CookieRequest {
        const ID: i32 = 0;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomPayload;

    impl Packet for CustomPayload {
        const ID: i32 = 1;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 2
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Disconnect;

    impl Packet for Disconnect {
        const ID: i32 = 2;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 3
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct FinishConfiguration;

    impl Packet for FinishConfiguration {
        const ID: i32 = 3;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 4
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct KeepAlive;

    impl Packet for KeepAlive {
        const ID: i32 = 4;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 5
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Ping;

    impl Packet for Ping {
        const ID: i32 = 5;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 6
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResetChat;

    impl Packet for ResetChat {
        const ID: i32 = 6;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 7
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RegistryData;

    impl Packet for RegistryData {
        const ID: i32 = 7;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 8
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePackPop;

    impl Packet for ResourcePackPop {
        const ID: i32 = 8;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 9
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePackPush;

    impl Packet for ResourcePackPush {
        const ID: i32 = 9;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 10
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StoreCookie;

    impl Packet for StoreCookie {
        const ID: i32 = 10;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 11
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Transfer;

    impl Packet for Transfer {
        const ID: i32 = 11;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 12
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateEnabledFeatures;

    impl Packet for UpdateEnabledFeatures {
        const ID: i32 = 12;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 13
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateTags;

    impl Packet for UpdateTags {
        const ID: i32 = 13;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 14
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SelectKnownPacks;

    impl Packet for SelectKnownPacks {
        const ID: i32 = 14;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 15
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomReportDetails;

    impl Packet for CustomReportDetails {
        const ID: i32 = 15;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 16
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ServerLinks;

    impl Packet for ServerLinks {
        const ID: i32 = 16;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 17
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClearDialog;

    impl Packet for ClearDialog {
        const ID: i32 = 17;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 18
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ShowDialog;

    impl Packet for ShowDialog {
        const ID: i32 = 18;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 19
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CodeOfConduct;

    impl Packet for CodeOfConduct {
        const ID: i32 = 19;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Clientbound;
    }

}

pub mod serverbound {
    use super::*;

    /// Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClientInformation;

    impl Packet for ClientInformation {
        const ID: i32 = 0;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieResponse;

    impl Packet for CookieResponse {
        const ID: i32 = 1;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 2
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomPayload;

    impl Packet for CustomPayload {
        const ID: i32 = 2;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 3
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct FinishConfiguration;

    impl Packet for FinishConfiguration {
        const ID: i32 = 3;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 4
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct KeepAlive;

    impl Packet for KeepAlive {
        const ID: i32 = 4;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 5
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Pong;

    impl Packet for Pong {
        const ID: i32 = 5;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 6
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePack;

    impl Packet for ResourcePack {
        const ID: i32 = 6;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 7
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SelectKnownPacks;

    impl Packet for SelectKnownPacks {
        const ID: i32 = 7;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 8
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomClickAction;

    impl Packet for CustomClickAction {
        const ID: i32 = 8;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 9
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct AcceptCodeOfConduct;

    impl Packet for AcceptCodeOfConduct {
        const ID: i32 = 9;
        const STATE: State = State::Configuration;
        const DIRECTION: Direction = Direction::Serverbound;
    }

}
