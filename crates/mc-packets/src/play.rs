// Auto-generated from Minecraft - play
// Do not edit manually

#![allow(dead_code)]

use std::borrow::Cow;
use mc_protocol::{Encode, Decode, Packet, State, Direction, VarInt, Uuid, Position, Nbt, BlockState};
use serde::{Serialize, Deserialize};

pub mod clientbound {
    use super::*;

    /// Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BundleDelimiter;

    impl Packet for BundleDelimiter {
        const ID: i32 = 0;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct AddEntity;

    impl Packet for AddEntity {
        const ID: i32 = 1;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 2
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Animate;

    impl Packet for Animate {
        const ID: i32 = 2;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 3
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct AwardStats;

    impl Packet for AwardStats {
        const ID: i32 = 3;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 4
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockChangedAck;

    impl Packet for BlockChangedAck {
        const ID: i32 = 4;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 5
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockDestruction;

    impl Packet for BlockDestruction {
        const ID: i32 = 5;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 6
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockEntityData;

    impl Packet for BlockEntityData {
        const ID: i32 = 6;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 7
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockEvent;

    impl Packet for BlockEvent {
        const ID: i32 = 7;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 8
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockUpdate;

    impl Packet for BlockUpdate {
        const ID: i32 = 8;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 9
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BossEvent;

    impl Packet for BossEvent {
        const ID: i32 = 9;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 10
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChangeDifficulty;

    impl Packet for ChangeDifficulty {
        const ID: i32 = 10;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 11
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChunkBatchFinished;

    impl Packet for ChunkBatchFinished {
        const ID: i32 = 11;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 12
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChunkBatchStart;

    impl Packet for ChunkBatchStart {
        const ID: i32 = 12;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 13
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChunksBiomes;

    impl Packet for ChunksBiomes {
        const ID: i32 = 13;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 14
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClearTitles;

    impl Packet for ClearTitles {
        const ID: i32 = 14;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 15
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CommandSuggestions;

    impl Packet for CommandSuggestions {
        const ID: i32 = 15;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 16
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Commands;

    impl Packet for Commands {
        const ID: i32 = 16;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 17
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerClose;

    impl Packet for ContainerClose {
        const ID: i32 = 17;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 18
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerSetContent;

    impl Packet for ContainerSetContent {
        const ID: i32 = 18;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 19
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerSetData;

    impl Packet for ContainerSetData {
        const ID: i32 = 19;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 20
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerSetSlot;

    impl Packet for ContainerSetSlot {
        const ID: i32 = 20;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 21
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieRequest;

    impl Packet for CookieRequest {
        const ID: i32 = 21;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 22
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Cooldown;

    impl Packet for Cooldown {
        const ID: i32 = 22;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 23
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomChatCompletions;

    impl Packet for CustomChatCompletions {
        const ID: i32 = 23;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 24
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomPayload;

    impl Packet for CustomPayload {
        const ID: i32 = 24;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 25
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DamageEvent;

    impl Packet for DamageEvent {
        const ID: i32 = 25;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 26
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugBlockValue;

    impl Packet for DebugBlockValue {
        const ID: i32 = 26;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 27
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugChunkValue;

    impl Packet for DebugChunkValue {
        const ID: i32 = 27;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 28
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugEntityValue;

    impl Packet for DebugEntityValue {
        const ID: i32 = 28;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 29
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugEvent;

    impl Packet for DebugEvent {
        const ID: i32 = 29;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 30
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugSample;

    impl Packet for DebugSample {
        const ID: i32 = 30;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 31
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DeleteChat;

    impl Packet for DeleteChat {
        const ID: i32 = 31;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 32
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Disconnect;

    impl Packet for Disconnect {
        const ID: i32 = 32;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 33
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DisguisedChat;

    impl Packet for DisguisedChat {
        const ID: i32 = 33;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 34
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct EntityEvent;

    impl Packet for EntityEvent {
        const ID: i32 = 34;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 35
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct EntityPositionSync;

    impl Packet for EntityPositionSync {
        const ID: i32 = 35;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 36
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Explode;

    impl Packet for Explode {
        const ID: i32 = 36;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 37
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ForgetLevelChunk;

    impl Packet for ForgetLevelChunk {
        const ID: i32 = 37;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 38
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct GameEvent;

    impl Packet for GameEvent {
        const ID: i32 = 38;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 39
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct GameTestHighlightPos;

    impl Packet for GameTestHighlightPos {
        const ID: i32 = 39;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 40
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MountScreenOpen;

    impl Packet for MountScreenOpen {
        const ID: i32 = 40;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 41
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct HurtAnimation;

    impl Packet for HurtAnimation {
        const ID: i32 = 41;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 42
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct InitializeBorder;

    impl Packet for InitializeBorder {
        const ID: i32 = 42;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 43
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct KeepAlive;

    impl Packet for KeepAlive {
        const ID: i32 = 43;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 44
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LevelChunkWithLight;

    impl Packet for LevelChunkWithLight {
        const ID: i32 = 44;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 45
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LevelEvent;

    impl Packet for LevelEvent {
        const ID: i32 = 45;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 46
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LevelParticles;

    impl Packet for LevelParticles {
        const ID: i32 = 46;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 47
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LightUpdate;

    impl Packet for LightUpdate {
        const ID: i32 = 47;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 48
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Login;

    impl Packet for Login {
        const ID: i32 = 48;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 49
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MapItemData;

    impl Packet for MapItemData {
        const ID: i32 = 49;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 50
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MerchantOffers;

    impl Packet for MerchantOffers {
        const ID: i32 = 50;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 51
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveEntityPos;

    impl Packet for MoveEntityPos {
        const ID: i32 = 51;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 52
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveEntityPosRot;

    impl Packet for MoveEntityPosRot {
        const ID: i32 = 52;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 53
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveMinecartAlongTrack;

    impl Packet for MoveMinecartAlongTrack {
        const ID: i32 = 53;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 54
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveEntityRot;

    impl Packet for MoveEntityRot {
        const ID: i32 = 54;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 55
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveVehicle;

    impl Packet for MoveVehicle {
        const ID: i32 = 55;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 56
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct OpenBook;

    impl Packet for OpenBook {
        const ID: i32 = 56;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 57
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct OpenScreen;

    impl Packet for OpenScreen {
        const ID: i32 = 57;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 58
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct OpenSignEditor;

    impl Packet for OpenSignEditor {
        const ID: i32 = 58;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 59
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Ping;

    impl Packet for Ping {
        const ID: i32 = 59;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 60
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PongResponse;

    impl Packet for PongResponse {
        const ID: i32 = 60;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 61
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlaceGhostRecipe;

    impl Packet for PlaceGhostRecipe {
        const ID: i32 = 61;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 62
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerAbilities;

    impl Packet for PlayerAbilities {
        const ID: i32 = 62;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 63
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerChat;

    impl Packet for PlayerChat {
        const ID: i32 = 63;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 64
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerCombatEnd;

    impl Packet for PlayerCombatEnd {
        const ID: i32 = 64;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 65
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerCombatEnter;

    impl Packet for PlayerCombatEnter {
        const ID: i32 = 65;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 66
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerCombatKill;

    impl Packet for PlayerCombatKill {
        const ID: i32 = 66;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 67
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerInfoRemove;

    impl Packet for PlayerInfoRemove {
        const ID: i32 = 67;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 68
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerInfoUpdate;

    impl Packet for PlayerInfoUpdate {
        const ID: i32 = 68;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 69
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerLookAt;

    impl Packet for PlayerLookAt {
        const ID: i32 = 69;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 70
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerPosition;

    impl Packet for PlayerPosition {
        const ID: i32 = 70;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 71
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerRotation;

    impl Packet for PlayerRotation {
        const ID: i32 = 71;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 72
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookAdd;

    impl Packet for RecipeBookAdd {
        const ID: i32 = 72;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 73
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookRemove;

    impl Packet for RecipeBookRemove {
        const ID: i32 = 73;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 74
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookSettings;

    impl Packet for RecipeBookSettings {
        const ID: i32 = 74;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 75
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RemoveEntities;

    impl Packet for RemoveEntities {
        const ID: i32 = 75;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 76
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RemoveMobEffect;

    impl Packet for RemoveMobEffect {
        const ID: i32 = 76;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 77
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResetScore;

    impl Packet for ResetScore {
        const ID: i32 = 77;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 78
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePackPop;

    impl Packet for ResourcePackPop {
        const ID: i32 = 78;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 79
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePackPush;

    impl Packet for ResourcePackPush {
        const ID: i32 = 79;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 80
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Respawn;

    impl Packet for Respawn {
        const ID: i32 = 80;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 81
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RotateHead;

    impl Packet for RotateHead {
        const ID: i32 = 81;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 82
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SectionBlocksUpdate;

    impl Packet for SectionBlocksUpdate {
        const ID: i32 = 82;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 83
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SelectAdvancementsTab;

    impl Packet for SelectAdvancementsTab {
        const ID: i32 = 83;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 84
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ServerData;

    impl Packet for ServerData {
        const ID: i32 = 84;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 85
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetActionBarText;

    impl Packet for SetActionBarText {
        const ID: i32 = 85;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 86
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderCenter;

    impl Packet for SetBorderCenter {
        const ID: i32 = 86;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 87
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderLerpSize;

    impl Packet for SetBorderLerpSize {
        const ID: i32 = 87;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 88
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderSize;

    impl Packet for SetBorderSize {
        const ID: i32 = 88;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 89
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderWarningDelay;

    impl Packet for SetBorderWarningDelay {
        const ID: i32 = 89;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 90
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderWarningDistance;

    impl Packet for SetBorderWarningDistance {
        const ID: i32 = 90;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 91
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCamera;

    impl Packet for SetCamera {
        const ID: i32 = 91;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 92
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetChunkCacheCenter;

    impl Packet for SetChunkCacheCenter {
        const ID: i32 = 92;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 93
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetChunkCacheRadius;

    impl Packet for SetChunkCacheRadius {
        const ID: i32 = 93;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 94
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCursorItem;

    impl Packet for SetCursorItem {
        const ID: i32 = 94;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 95
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetDefaultSpawnPosition;

    impl Packet for SetDefaultSpawnPosition {
        const ID: i32 = 95;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 96
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetDisplayObjective;

    impl Packet for SetDisplayObjective {
        const ID: i32 = 96;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 97
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetEntityData;

    impl Packet for SetEntityData {
        const ID: i32 = 97;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 98
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetEntityLink;

    impl Packet for SetEntityLink {
        const ID: i32 = 98;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 99
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetEntityMotion;

    impl Packet for SetEntityMotion {
        const ID: i32 = 99;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 100
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetEquipment;

    impl Packet for SetEquipment {
        const ID: i32 = 100;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 101
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetExperience;

    impl Packet for SetExperience {
        const ID: i32 = 101;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 102
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetHealth;

    impl Packet for SetHealth {
        const ID: i32 = 102;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 103
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetHeldSlot;

    impl Packet for SetHeldSlot {
        const ID: i32 = 103;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 104
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetObjective;

    impl Packet for SetObjective {
        const ID: i32 = 104;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 105
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetPassengers;

    impl Packet for SetPassengers {
        const ID: i32 = 105;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 106
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetPlayerInventory;

    impl Packet for SetPlayerInventory {
        const ID: i32 = 106;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 107
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetPlayerTeam;

    impl Packet for SetPlayerTeam {
        const ID: i32 = 107;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 108
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetScore;

    impl Packet for SetScore {
        const ID: i32 = 108;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 109
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetSimulationDistance;

    impl Packet for SetSimulationDistance {
        const ID: i32 = 109;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 110
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetSubtitleText;

    impl Packet for SetSubtitleText {
        const ID: i32 = 110;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 111
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetTime;

    impl Packet for SetTime {
        const ID: i32 = 111;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 112
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetTitleText;

    impl Packet for SetTitleText {
        const ID: i32 = 112;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 113
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetTitlesAnimation;

    impl Packet for SetTitlesAnimation {
        const ID: i32 = 113;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 114
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SoundEntity;

    impl Packet for SoundEntity {
        const ID: i32 = 114;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 115
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Sound;

    impl Packet for Sound {
        const ID: i32 = 115;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 116
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StartConfiguration;

    impl Packet for StartConfiguration {
        const ID: i32 = 116;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 117
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StopSound;

    impl Packet for StopSound {
        const ID: i32 = 117;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 118
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StoreCookie;

    impl Packet for StoreCookie {
        const ID: i32 = 118;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 119
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SystemChat;

    impl Packet for SystemChat {
        const ID: i32 = 119;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 120
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TabList;

    impl Packet for TabList {
        const ID: i32 = 120;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 121
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TagQuery;

    impl Packet for TagQuery {
        const ID: i32 = 121;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 122
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TakeItemEntity;

    impl Packet for TakeItemEntity {
        const ID: i32 = 122;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 123
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TeleportEntity;

    impl Packet for TeleportEntity {
        const ID: i32 = 123;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 124
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TestInstanceBlockStatus;

    impl Packet for TestInstanceBlockStatus {
        const ID: i32 = 124;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 125
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TickingState;

    impl Packet for TickingState {
        const ID: i32 = 125;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 126
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TickingStep;

    impl Packet for TickingStep {
        const ID: i32 = 126;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 127
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Transfer;

    impl Packet for Transfer {
        const ID: i32 = 127;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 128
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateAdvancements;

    impl Packet for UpdateAdvancements {
        const ID: i32 = 128;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 129
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateAttributes;

    impl Packet for UpdateAttributes {
        const ID: i32 = 129;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 130
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateMobEffect;

    impl Packet for UpdateMobEffect {
        const ID: i32 = 130;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 131
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateRecipes;

    impl Packet for UpdateRecipes {
        const ID: i32 = 131;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 132
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateTags;

    impl Packet for UpdateTags {
        const ID: i32 = 132;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 133
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ProjectilePower;

    impl Packet for ProjectilePower {
        const ID: i32 = 133;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 134
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomReportDetails;

    impl Packet for CustomReportDetails {
        const ID: i32 = 134;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 135
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ServerLinks;

    impl Packet for ServerLinks {
        const ID: i32 = 135;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 136
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Waypoint;

    impl Packet for Waypoint {
        const ID: i32 = 136;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 137
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClearDialog;

    impl Packet for ClearDialog {
        const ID: i32 = 137;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

    /// Packet ID: 138
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ShowDialog;

    impl Packet for ShowDialog {
        const ID: i32 = 138;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Clientbound;
    }

}

pub mod serverbound {
    use super::*;

    /// Packet ID: 0
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct AcceptTeleportation;

    impl Packet for AcceptTeleportation {
        const ID: i32 = 0;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 1
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockEntityTagQuery;

    impl Packet for BlockEntityTagQuery {
        const ID: i32 = 1;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 2
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BundleItemSelected;

    impl Packet for BundleItemSelected {
        const ID: i32 = 2;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 3
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChangeDifficulty;

    impl Packet for ChangeDifficulty {
        const ID: i32 = 3;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 4
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChangeGameMode;

    impl Packet for ChangeGameMode {
        const ID: i32 = 4;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 5
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChatAck;

    impl Packet for ChatAck {
        const ID: i32 = 5;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 6
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChatCommand;

    impl Packet for ChatCommand {
        const ID: i32 = 6;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 7
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChatCommandSigned;

    impl Packet for ChatCommandSigned {
        const ID: i32 = 7;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 8
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Chat;

    impl Packet for Chat {
        const ID: i32 = 8;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 9
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChatSessionUpdate;

    impl Packet for ChatSessionUpdate {
        const ID: i32 = 9;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 10
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChunkBatchReceived;

    impl Packet for ChunkBatchReceived {
        const ID: i32 = 10;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 11
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClientCommand;

    impl Packet for ClientCommand {
        const ID: i32 = 11;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 12
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClientTickEnd;

    impl Packet for ClientTickEnd {
        const ID: i32 = 12;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 13
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClientInformation;

    impl Packet for ClientInformation {
        const ID: i32 = 13;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 14
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CommandSuggestion;

    impl Packet for CommandSuggestion {
        const ID: i32 = 14;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 15
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ConfigurationAcknowledged;

    impl Packet for ConfigurationAcknowledged {
        const ID: i32 = 15;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 16
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerButtonClick;

    impl Packet for ContainerButtonClick {
        const ID: i32 = 16;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 17
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerClick;

    impl Packet for ContainerClick {
        const ID: i32 = 17;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 18
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerClose;

    impl Packet for ContainerClose {
        const ID: i32 = 18;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 19
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerSlotStateChanged;

    impl Packet for ContainerSlotStateChanged {
        const ID: i32 = 19;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 20
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieResponse;

    impl Packet for CookieResponse {
        const ID: i32 = 20;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 21
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomPayload;

    impl Packet for CustomPayload {
        const ID: i32 = 21;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 22
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugSubscriptionRequest;

    impl Packet for DebugSubscriptionRequest {
        const ID: i32 = 22;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 23
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct EditBook;

    impl Packet for EditBook {
        const ID: i32 = 23;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 24
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct EntityTagQuery;

    impl Packet for EntityTagQuery {
        const ID: i32 = 24;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 25
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Interact;

    impl Packet for Interact {
        const ID: i32 = 25;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 26
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct JigsawGenerate;

    impl Packet for JigsawGenerate {
        const ID: i32 = 26;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 27
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct KeepAlive;

    impl Packet for KeepAlive {
        const ID: i32 = 27;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 28
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LockDifficulty;

    impl Packet for LockDifficulty {
        const ID: i32 = 28;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 29
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MovePlayerPos;

    impl Packet for MovePlayerPos {
        const ID: i32 = 29;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 30
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MovePlayerPosRot;

    impl Packet for MovePlayerPosRot {
        const ID: i32 = 30;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 31
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MovePlayerRot;

    impl Packet for MovePlayerRot {
        const ID: i32 = 31;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 32
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MovePlayerStatusOnly;

    impl Packet for MovePlayerStatusOnly {
        const ID: i32 = 32;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 33
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveVehicle;

    impl Packet for MoveVehicle {
        const ID: i32 = 33;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 34
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PaddleBoat;

    impl Packet for PaddleBoat {
        const ID: i32 = 34;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 35
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PickItemFromBlock;

    impl Packet for PickItemFromBlock {
        const ID: i32 = 35;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 36
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PickItemFromEntity;

    impl Packet for PickItemFromEntity {
        const ID: i32 = 36;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 37
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PingRequest;

    impl Packet for PingRequest {
        const ID: i32 = 37;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 38
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlaceRecipe;

    impl Packet for PlaceRecipe {
        const ID: i32 = 38;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 39
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerAbilities;

    impl Packet for PlayerAbilities {
        const ID: i32 = 39;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 40
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerAction;

    impl Packet for PlayerAction {
        const ID: i32 = 40;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 41
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerCommand;

    impl Packet for PlayerCommand {
        const ID: i32 = 41;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 42
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerInput;

    impl Packet for PlayerInput {
        const ID: i32 = 42;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 43
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerLoaded;

    impl Packet for PlayerLoaded {
        const ID: i32 = 43;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 44
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Pong;

    impl Packet for Pong {
        const ID: i32 = 44;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 45
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookChangeSettings;

    impl Packet for RecipeBookChangeSettings {
        const ID: i32 = 45;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 46
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookSeenRecipe;

    impl Packet for RecipeBookSeenRecipe {
        const ID: i32 = 46;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 47
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RenameItem;

    impl Packet for RenameItem {
        const ID: i32 = 47;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 48
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePack;

    impl Packet for ResourcePack {
        const ID: i32 = 48;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 49
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SeenAdvancements;

    impl Packet for SeenAdvancements {
        const ID: i32 = 49;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 50
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SelectTrade;

    impl Packet for SelectTrade {
        const ID: i32 = 50;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 51
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBeacon;

    impl Packet for SetBeacon {
        const ID: i32 = 51;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 52
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCarriedItem;

    impl Packet for SetCarriedItem {
        const ID: i32 = 52;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 53
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCommandBlock;

    impl Packet for SetCommandBlock {
        const ID: i32 = 53;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 54
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCommandMinecart;

    impl Packet for SetCommandMinecart {
        const ID: i32 = 54;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 55
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCreativeModeSlot;

    impl Packet for SetCreativeModeSlot {
        const ID: i32 = 55;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 56
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetJigsawBlock;

    impl Packet for SetJigsawBlock {
        const ID: i32 = 56;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 57
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetStructureBlock;

    impl Packet for SetStructureBlock {
        const ID: i32 = 57;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 58
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetTestBlock;

    impl Packet for SetTestBlock {
        const ID: i32 = 58;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 59
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SignUpdate;

    impl Packet for SignUpdate {
        const ID: i32 = 59;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 60
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Swing;

    impl Packet for Swing {
        const ID: i32 = 60;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 61
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TeleportToEntity;

    impl Packet for TeleportToEntity {
        const ID: i32 = 61;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 62
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TestInstanceBlockAction;

    impl Packet for TestInstanceBlockAction {
        const ID: i32 = 62;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 63
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UseItemOn;

    impl Packet for UseItemOn {
        const ID: i32 = 63;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 64
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UseItem;

    impl Packet for UseItem {
        const ID: i32 = 64;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

    /// Packet ID: 65
    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomClickAction;

    impl Packet for CustomClickAction {
        const ID: i32 = 65;
        const STATE: State = State::Play;
        const DIRECTION: Direction = Direction::Serverbound;
    }

}
