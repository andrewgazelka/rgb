// Auto-generated from Minecraft - configuration
// Do not edit manually

#![allow(dead_code)]

use std::borrow::Cow;
use mc_protocol::{Encode, Decode, VarInt, Uuid, Position, Nbt, BlockState};
use serde::{Serialize, Deserialize};

pub mod clientbound {
    use super::*;

    /// CookieRequest (ID: 0)
    pub const COOKIE_REQUEST_ID: i32 = 0;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieRequest;

    /// CustomPayload (ID: 1)
    pub const CUSTOM_PAYLOAD_ID: i32 = 1;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomPayload;

    /// Disconnect (ID: 2)
    pub const DISCONNECT_ID: i32 = 2;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Disconnect;

    /// FinishConfiguration (ID: 3)
    pub const FINISH_CONFIGURATION_ID: i32 = 3;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct FinishConfiguration;

    /// KeepAlive (ID: 4)
    pub const KEEP_ALIVE_ID: i32 = 4;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct KeepAlive;

    /// Ping (ID: 5)
    pub const PING_ID: i32 = 5;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Ping;

    /// ResetChat (ID: 6)
    pub const RESET_CHAT_ID: i32 = 6;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResetChat;

    /// RegistryData (ID: 7)
    pub const REGISTRY_DATA_ID: i32 = 7;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RegistryData;

    /// ResourcePackPop (ID: 8)
    pub const RESOURCE_PACK_POP_ID: i32 = 8;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePackPop;

    /// ResourcePackPush (ID: 9)
    pub const RESOURCE_PACK_PUSH_ID: i32 = 9;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePackPush;

    /// StoreCookie (ID: 10)
    pub const STORE_COOKIE_ID: i32 = 10;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StoreCookie;

    /// Transfer (ID: 11)
    pub const TRANSFER_ID: i32 = 11;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Transfer;

    /// UpdateEnabledFeatures (ID: 12)
    pub const UPDATE_ENABLED_FEATURES_ID: i32 = 12;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateEnabledFeatures;

    /// UpdateTags (ID: 13)
    pub const UPDATE_TAGS_ID: i32 = 13;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateTags;

    /// SelectKnownPacks (ID: 14)
    pub const SELECT_KNOWN_PACKS_ID: i32 = 14;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SelectKnownPacks;

    /// CustomReportDetails (ID: 15)
    pub const CUSTOM_REPORT_DETAILS_ID: i32 = 15;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomReportDetails;

    /// ServerLinks (ID: 16)
    pub const SERVER_LINKS_ID: i32 = 16;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ServerLinks;

    /// ClearDialog (ID: 17)
    pub const CLEAR_DIALOG_ID: i32 = 17;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClearDialog;

    /// ShowDialog (ID: 18)
    pub const SHOW_DIALOG_ID: i32 = 18;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ShowDialog;

    /// CodeOfConduct (ID: 19)
    pub const CODE_OF_CONDUCT_ID: i32 = 19;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CodeOfConduct;

}

pub mod serverbound {
    use super::*;

    /// ClientInformation (ID: 0)
    pub const CLIENT_INFORMATION_ID: i32 = 0;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClientInformation;

    /// CookieResponse (ID: 1)
    pub const COOKIE_RESPONSE_ID: i32 = 1;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieResponse;

    /// CustomPayload (ID: 2)
    pub const CUSTOM_PAYLOAD_ID: i32 = 2;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomPayload;

    /// FinishConfiguration (ID: 3)
    pub const FINISH_CONFIGURATION_ID: i32 = 3;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct FinishConfiguration;

    /// KeepAlive (ID: 4)
    pub const KEEP_ALIVE_ID: i32 = 4;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct KeepAlive;

    /// Pong (ID: 5)
    pub const PONG_ID: i32 = 5;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Pong;

    /// ResourcePack (ID: 6)
    pub const RESOURCE_PACK_ID: i32 = 6;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePack;

    /// SelectKnownPacks (ID: 7)
    pub const SELECT_KNOWN_PACKS_ID: i32 = 7;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SelectKnownPacks;

    /// CustomClickAction (ID: 8)
    pub const CUSTOM_CLICK_ACTION_ID: i32 = 8;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomClickAction;

    /// AcceptCodeOfConduct (ID: 9)
    pub const ACCEPT_CODE_OF_CONDUCT_ID: i32 = 9;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct AcceptCodeOfConduct;

}
