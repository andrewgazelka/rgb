// Auto-generated from Minecraft - play
// Do not edit manually

#![allow(dead_code)]

use std::borrow::Cow;
use mc_protocol::{Encode, Decode, VarInt, Uuid, Position, Nbt, BlockState};
use serde::{Serialize, Deserialize};

pub mod clientbound {
    use super::*;

    /// BundleDelimiter (ID: 0)
    pub const BUNDLE_DELIMITER_ID: i32 = 0;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BundleDelimiter;

    /// AddEntity (ID: 1)
    pub const ADD_ENTITY_ID: i32 = 1;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct AddEntity;

    /// Animate (ID: 2)
    pub const ANIMATE_ID: i32 = 2;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Animate;

    /// AwardStats (ID: 3)
    pub const AWARD_STATS_ID: i32 = 3;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct AwardStats;

    /// BlockChangedAck (ID: 4)
    pub const BLOCK_CHANGED_ACK_ID: i32 = 4;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockChangedAck;

    /// BlockDestruction (ID: 5)
    pub const BLOCK_DESTRUCTION_ID: i32 = 5;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockDestruction;

    /// BlockEntityData (ID: 6)
    pub const BLOCK_ENTITY_DATA_ID: i32 = 6;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockEntityData;

    /// BlockEvent (ID: 7)
    pub const BLOCK_EVENT_ID: i32 = 7;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockEvent;

    /// BlockUpdate (ID: 8)
    pub const BLOCK_UPDATE_ID: i32 = 8;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockUpdate;

    /// BossEvent (ID: 9)
    pub const BOSS_EVENT_ID: i32 = 9;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BossEvent;

    /// ChangeDifficulty (ID: 10)
    pub const CHANGE_DIFFICULTY_ID: i32 = 10;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChangeDifficulty;

    /// ChunkBatchFinished (ID: 11)
    pub const CHUNK_BATCH_FINISHED_ID: i32 = 11;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChunkBatchFinished;

    /// ChunkBatchStart (ID: 12)
    pub const CHUNK_BATCH_START_ID: i32 = 12;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChunkBatchStart;

    /// ChunksBiomes (ID: 13)
    pub const CHUNKS_BIOMES_ID: i32 = 13;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChunksBiomes;

    /// ClearTitles (ID: 14)
    pub const CLEAR_TITLES_ID: i32 = 14;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClearTitles;

    /// CommandSuggestions (ID: 15)
    pub const COMMAND_SUGGESTIONS_ID: i32 = 15;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CommandSuggestions;

    /// Commands (ID: 16)
    pub const COMMANDS_ID: i32 = 16;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Commands;

    /// ContainerClose (ID: 17)
    pub const CONTAINER_CLOSE_ID: i32 = 17;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerClose;

    /// ContainerSetContent (ID: 18)
    pub const CONTAINER_SET_CONTENT_ID: i32 = 18;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerSetContent;

    /// ContainerSetData (ID: 19)
    pub const CONTAINER_SET_DATA_ID: i32 = 19;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerSetData;

    /// ContainerSetSlot (ID: 20)
    pub const CONTAINER_SET_SLOT_ID: i32 = 20;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerSetSlot;

    /// CookieRequest (ID: 21)
    pub const COOKIE_REQUEST_ID: i32 = 21;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieRequest;

    /// Cooldown (ID: 22)
    pub const COOLDOWN_ID: i32 = 22;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Cooldown;

    /// CustomChatCompletions (ID: 23)
    pub const CUSTOM_CHAT_COMPLETIONS_ID: i32 = 23;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomChatCompletions;

    /// CustomPayload (ID: 24)
    pub const CUSTOM_PAYLOAD_ID: i32 = 24;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomPayload;

    /// DamageEvent (ID: 25)
    pub const DAMAGE_EVENT_ID: i32 = 25;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DamageEvent;

    /// DebugBlockValue (ID: 26)
    pub const DEBUG_BLOCK_VALUE_ID: i32 = 26;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugBlockValue;

    /// DebugChunkValue (ID: 27)
    pub const DEBUG_CHUNK_VALUE_ID: i32 = 27;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugChunkValue;

    /// DebugEntityValue (ID: 28)
    pub const DEBUG_ENTITY_VALUE_ID: i32 = 28;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugEntityValue;

    /// DebugEvent (ID: 29)
    pub const DEBUG_EVENT_ID: i32 = 29;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugEvent;

    /// DebugSample (ID: 30)
    pub const DEBUG_SAMPLE_ID: i32 = 30;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugSample;

    /// DeleteChat (ID: 31)
    pub const DELETE_CHAT_ID: i32 = 31;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DeleteChat;

    /// Disconnect (ID: 32)
    pub const DISCONNECT_ID: i32 = 32;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Disconnect;

    /// DisguisedChat (ID: 33)
    pub const DISGUISED_CHAT_ID: i32 = 33;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DisguisedChat;

    /// EntityEvent (ID: 34)
    pub const ENTITY_EVENT_ID: i32 = 34;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct EntityEvent;

    /// EntityPositionSync (ID: 35)
    pub const ENTITY_POSITION_SYNC_ID: i32 = 35;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct EntityPositionSync;

    /// Explode (ID: 36)
    pub const EXPLODE_ID: i32 = 36;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Explode;

    /// ForgetLevelChunk (ID: 37)
    pub const FORGET_LEVEL_CHUNK_ID: i32 = 37;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ForgetLevelChunk;

    /// GameEvent (ID: 38)
    pub const GAME_EVENT_ID: i32 = 38;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct GameEvent;

    /// GameTestHighlightPos (ID: 39)
    pub const GAME_TEST_HIGHLIGHT_POS_ID: i32 = 39;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct GameTestHighlightPos;

    /// MountScreenOpen (ID: 40)
    pub const MOUNT_SCREEN_OPEN_ID: i32 = 40;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MountScreenOpen;

    /// HurtAnimation (ID: 41)
    pub const HURT_ANIMATION_ID: i32 = 41;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct HurtAnimation;

    /// InitializeBorder (ID: 42)
    pub const INITIALIZE_BORDER_ID: i32 = 42;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct InitializeBorder;

    /// KeepAlive (ID: 43)
    pub const KEEP_ALIVE_ID: i32 = 43;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct KeepAlive;

    /// LevelChunkWithLight (ID: 44)
    pub const LEVEL_CHUNK_WITH_LIGHT_ID: i32 = 44;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LevelChunkWithLight;

    /// LevelEvent (ID: 45)
    pub const LEVEL_EVENT_ID: i32 = 45;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LevelEvent;

    /// LevelParticles (ID: 46)
    pub const LEVEL_PARTICLES_ID: i32 = 46;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LevelParticles;

    /// LightUpdate (ID: 47)
    pub const LIGHT_UPDATE_ID: i32 = 47;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LightUpdate;

    /// Login (ID: 48)
    pub const LOGIN_ID: i32 = 48;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Login;

    /// MapItemData (ID: 49)
    pub const MAP_ITEM_DATA_ID: i32 = 49;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MapItemData;

    /// MerchantOffers (ID: 50)
    pub const MERCHANT_OFFERS_ID: i32 = 50;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MerchantOffers;

    /// MoveEntityPos (ID: 51)
    pub const MOVE_ENTITY_POS_ID: i32 = 51;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveEntityPos;

    /// MoveEntityPosRot (ID: 52)
    pub const MOVE_ENTITY_POS_ROT_ID: i32 = 52;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveEntityPosRot;

    /// MoveMinecartAlongTrack (ID: 53)
    pub const MOVE_MINECART_ALONG_TRACK_ID: i32 = 53;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveMinecartAlongTrack;

    /// MoveEntityRot (ID: 54)
    pub const MOVE_ENTITY_ROT_ID: i32 = 54;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveEntityRot;

    /// MoveVehicle (ID: 55)
    pub const MOVE_VEHICLE_ID: i32 = 55;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveVehicle;

    /// OpenBook (ID: 56)
    pub const OPEN_BOOK_ID: i32 = 56;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct OpenBook;

    /// OpenScreen (ID: 57)
    pub const OPEN_SCREEN_ID: i32 = 57;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct OpenScreen;

    /// OpenSignEditor (ID: 58)
    pub const OPEN_SIGN_EDITOR_ID: i32 = 58;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct OpenSignEditor;

    /// Ping (ID: 59)
    pub const PING_ID: i32 = 59;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Ping;

    /// PongResponse (ID: 60)
    pub const PONG_RESPONSE_ID: i32 = 60;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PongResponse;

    /// PlaceGhostRecipe (ID: 61)
    pub const PLACE_GHOST_RECIPE_ID: i32 = 61;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlaceGhostRecipe;

    /// PlayerAbilities (ID: 62)
    pub const PLAYER_ABILITIES_ID: i32 = 62;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerAbilities;

    /// PlayerChat (ID: 63)
    pub const PLAYER_CHAT_ID: i32 = 63;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerChat;

    /// PlayerCombatEnd (ID: 64)
    pub const PLAYER_COMBAT_END_ID: i32 = 64;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerCombatEnd;

    /// PlayerCombatEnter (ID: 65)
    pub const PLAYER_COMBAT_ENTER_ID: i32 = 65;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerCombatEnter;

    /// PlayerCombatKill (ID: 66)
    pub const PLAYER_COMBAT_KILL_ID: i32 = 66;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerCombatKill;

    /// PlayerInfoRemove (ID: 67)
    pub const PLAYER_INFO_REMOVE_ID: i32 = 67;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerInfoRemove;

    /// PlayerInfoUpdate (ID: 68)
    pub const PLAYER_INFO_UPDATE_ID: i32 = 68;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerInfoUpdate;

    /// PlayerLookAt (ID: 69)
    pub const PLAYER_LOOK_AT_ID: i32 = 69;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerLookAt;

    /// PlayerPosition (ID: 70)
    pub const PLAYER_POSITION_ID: i32 = 70;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerPosition;

    /// PlayerRotation (ID: 71)
    pub const PLAYER_ROTATION_ID: i32 = 71;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerRotation;

    /// RecipeBookAdd (ID: 72)
    pub const RECIPE_BOOK_ADD_ID: i32 = 72;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookAdd;

    /// RecipeBookRemove (ID: 73)
    pub const RECIPE_BOOK_REMOVE_ID: i32 = 73;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookRemove;

    /// RecipeBookSettings (ID: 74)
    pub const RECIPE_BOOK_SETTINGS_ID: i32 = 74;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookSettings;

    /// RemoveEntities (ID: 75)
    pub const REMOVE_ENTITIES_ID: i32 = 75;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RemoveEntities;

    /// RemoveMobEffect (ID: 76)
    pub const REMOVE_MOB_EFFECT_ID: i32 = 76;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RemoveMobEffect;

    /// ResetScore (ID: 77)
    pub const RESET_SCORE_ID: i32 = 77;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResetScore;

    /// ResourcePackPop (ID: 78)
    pub const RESOURCE_PACK_POP_ID: i32 = 78;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePackPop;

    /// ResourcePackPush (ID: 79)
    pub const RESOURCE_PACK_PUSH_ID: i32 = 79;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePackPush;

    /// Respawn (ID: 80)
    pub const RESPAWN_ID: i32 = 80;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Respawn;

    /// RotateHead (ID: 81)
    pub const ROTATE_HEAD_ID: i32 = 81;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RotateHead;

    /// SectionBlocksUpdate (ID: 82)
    pub const SECTION_BLOCKS_UPDATE_ID: i32 = 82;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SectionBlocksUpdate;

    /// SelectAdvancementsTab (ID: 83)
    pub const SELECT_ADVANCEMENTS_TAB_ID: i32 = 83;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SelectAdvancementsTab;

    /// ServerData (ID: 84)
    pub const SERVER_DATA_ID: i32 = 84;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ServerData;

    /// SetActionBarText (ID: 85)
    pub const SET_ACTION_BAR_TEXT_ID: i32 = 85;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetActionBarText;

    /// SetBorderCenter (ID: 86)
    pub const SET_BORDER_CENTER_ID: i32 = 86;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderCenter;

    /// SetBorderLerpSize (ID: 87)
    pub const SET_BORDER_LERP_SIZE_ID: i32 = 87;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderLerpSize;

    /// SetBorderSize (ID: 88)
    pub const SET_BORDER_SIZE_ID: i32 = 88;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderSize;

    /// SetBorderWarningDelay (ID: 89)
    pub const SET_BORDER_WARNING_DELAY_ID: i32 = 89;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderWarningDelay;

    /// SetBorderWarningDistance (ID: 90)
    pub const SET_BORDER_WARNING_DISTANCE_ID: i32 = 90;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBorderWarningDistance;

    /// SetCamera (ID: 91)
    pub const SET_CAMERA_ID: i32 = 91;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCamera;

    /// SetChunkCacheCenter (ID: 92)
    pub const SET_CHUNK_CACHE_CENTER_ID: i32 = 92;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetChunkCacheCenter;

    /// SetChunkCacheRadius (ID: 93)
    pub const SET_CHUNK_CACHE_RADIUS_ID: i32 = 93;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetChunkCacheRadius;

    /// SetCursorItem (ID: 94)
    pub const SET_CURSOR_ITEM_ID: i32 = 94;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCursorItem;

    /// SetDefaultSpawnPosition (ID: 95)
    pub const SET_DEFAULT_SPAWN_POSITION_ID: i32 = 95;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetDefaultSpawnPosition;

    /// SetDisplayObjective (ID: 96)
    pub const SET_DISPLAY_OBJECTIVE_ID: i32 = 96;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetDisplayObjective;

    /// SetEntityData (ID: 97)
    pub const SET_ENTITY_DATA_ID: i32 = 97;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetEntityData;

    /// SetEntityLink (ID: 98)
    pub const SET_ENTITY_LINK_ID: i32 = 98;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetEntityLink;

    /// SetEntityMotion (ID: 99)
    pub const SET_ENTITY_MOTION_ID: i32 = 99;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetEntityMotion;

    /// SetEquipment (ID: 100)
    pub const SET_EQUIPMENT_ID: i32 = 100;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetEquipment;

    /// SetExperience (ID: 101)
    pub const SET_EXPERIENCE_ID: i32 = 101;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetExperience;

    /// SetHealth (ID: 102)
    pub const SET_HEALTH_ID: i32 = 102;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetHealth;

    /// SetHeldSlot (ID: 103)
    pub const SET_HELD_SLOT_ID: i32 = 103;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetHeldSlot;

    /// SetObjective (ID: 104)
    pub const SET_OBJECTIVE_ID: i32 = 104;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetObjective;

    /// SetPassengers (ID: 105)
    pub const SET_PASSENGERS_ID: i32 = 105;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetPassengers;

    /// SetPlayerInventory (ID: 106)
    pub const SET_PLAYER_INVENTORY_ID: i32 = 106;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetPlayerInventory;

    /// SetPlayerTeam (ID: 107)
    pub const SET_PLAYER_TEAM_ID: i32 = 107;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetPlayerTeam;

    /// SetScore (ID: 108)
    pub const SET_SCORE_ID: i32 = 108;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetScore;

    /// SetSimulationDistance (ID: 109)
    pub const SET_SIMULATION_DISTANCE_ID: i32 = 109;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetSimulationDistance;

    /// SetSubtitleText (ID: 110)
    pub const SET_SUBTITLE_TEXT_ID: i32 = 110;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetSubtitleText;

    /// SetTime (ID: 111)
    pub const SET_TIME_ID: i32 = 111;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetTime;

    /// SetTitleText (ID: 112)
    pub const SET_TITLE_TEXT_ID: i32 = 112;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetTitleText;

    /// SetTitlesAnimation (ID: 113)
    pub const SET_TITLES_ANIMATION_ID: i32 = 113;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetTitlesAnimation;

    /// SoundEntity (ID: 114)
    pub const SOUND_ENTITY_ID: i32 = 114;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SoundEntity;

    /// Sound (ID: 115)
    pub const SOUND_ID: i32 = 115;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Sound;

    /// StartConfiguration (ID: 116)
    pub const START_CONFIGURATION_ID: i32 = 116;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StartConfiguration;

    /// StopSound (ID: 117)
    pub const STOP_SOUND_ID: i32 = 117;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StopSound;

    /// StoreCookie (ID: 118)
    pub const STORE_COOKIE_ID: i32 = 118;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct StoreCookie;

    /// SystemChat (ID: 119)
    pub const SYSTEM_CHAT_ID: i32 = 119;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SystemChat;

    /// TabList (ID: 120)
    pub const TAB_LIST_ID: i32 = 120;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TabList;

    /// TagQuery (ID: 121)
    pub const TAG_QUERY_ID: i32 = 121;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TagQuery;

    /// TakeItemEntity (ID: 122)
    pub const TAKE_ITEM_ENTITY_ID: i32 = 122;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TakeItemEntity;

    /// TeleportEntity (ID: 123)
    pub const TELEPORT_ENTITY_ID: i32 = 123;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TeleportEntity;

    /// TestInstanceBlockStatus (ID: 124)
    pub const TEST_INSTANCE_BLOCK_STATUS_ID: i32 = 124;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TestInstanceBlockStatus;

    /// TickingState (ID: 125)
    pub const TICKING_STATE_ID: i32 = 125;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TickingState;

    /// TickingStep (ID: 126)
    pub const TICKING_STEP_ID: i32 = 126;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TickingStep;

    /// Transfer (ID: 127)
    pub const TRANSFER_ID: i32 = 127;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Transfer;

    /// UpdateAdvancements (ID: 128)
    pub const UPDATE_ADVANCEMENTS_ID: i32 = 128;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateAdvancements;

    /// UpdateAttributes (ID: 129)
    pub const UPDATE_ATTRIBUTES_ID: i32 = 129;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateAttributes;

    /// UpdateMobEffect (ID: 130)
    pub const UPDATE_MOB_EFFECT_ID: i32 = 130;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateMobEffect;

    /// UpdateRecipes (ID: 131)
    pub const UPDATE_RECIPES_ID: i32 = 131;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateRecipes;

    /// UpdateTags (ID: 132)
    pub const UPDATE_TAGS_ID: i32 = 132;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UpdateTags;

    /// ProjectilePower (ID: 133)
    pub const PROJECTILE_POWER_ID: i32 = 133;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ProjectilePower;

    /// CustomReportDetails (ID: 134)
    pub const CUSTOM_REPORT_DETAILS_ID: i32 = 134;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomReportDetails;

    /// ServerLinks (ID: 135)
    pub const SERVER_LINKS_ID: i32 = 135;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ServerLinks;

    /// Waypoint (ID: 136)
    pub const WAYPOINT_ID: i32 = 136;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Waypoint;

    /// ClearDialog (ID: 137)
    pub const CLEAR_DIALOG_ID: i32 = 137;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClearDialog;

    /// ShowDialog (ID: 138)
    pub const SHOW_DIALOG_ID: i32 = 138;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ShowDialog;

}

pub mod serverbound {
    use super::*;

    /// AcceptTeleportation (ID: 0)
    pub const ACCEPT_TELEPORTATION_ID: i32 = 0;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct AcceptTeleportation;

    /// BlockEntityTagQuery (ID: 1)
    pub const BLOCK_ENTITY_TAG_QUERY_ID: i32 = 1;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BlockEntityTagQuery;

    /// BundleItemSelected (ID: 2)
    pub const BUNDLE_ITEM_SELECTED_ID: i32 = 2;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct BundleItemSelected;

    /// ChangeDifficulty (ID: 3)
    pub const CHANGE_DIFFICULTY_ID: i32 = 3;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChangeDifficulty;

    /// ChangeGameMode (ID: 4)
    pub const CHANGE_GAME_MODE_ID: i32 = 4;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChangeGameMode;

    /// ChatAck (ID: 5)
    pub const CHAT_ACK_ID: i32 = 5;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChatAck;

    /// ChatCommand (ID: 6)
    pub const CHAT_COMMAND_ID: i32 = 6;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChatCommand;

    /// ChatCommandSigned (ID: 7)
    pub const CHAT_COMMAND_SIGNED_ID: i32 = 7;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChatCommandSigned;

    /// Chat (ID: 8)
    pub const CHAT_ID: i32 = 8;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Chat;

    /// ChatSessionUpdate (ID: 9)
    pub const CHAT_SESSION_UPDATE_ID: i32 = 9;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChatSessionUpdate;

    /// ChunkBatchReceived (ID: 10)
    pub const CHUNK_BATCH_RECEIVED_ID: i32 = 10;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ChunkBatchReceived;

    /// ClientCommand (ID: 11)
    pub const CLIENT_COMMAND_ID: i32 = 11;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClientCommand;

    /// ClientTickEnd (ID: 12)
    pub const CLIENT_TICK_END_ID: i32 = 12;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClientTickEnd;

    /// ClientInformation (ID: 13)
    pub const CLIENT_INFORMATION_ID: i32 = 13;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ClientInformation;

    /// CommandSuggestion (ID: 14)
    pub const COMMAND_SUGGESTION_ID: i32 = 14;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CommandSuggestion;

    /// ConfigurationAcknowledged (ID: 15)
    pub const CONFIGURATION_ACKNOWLEDGED_ID: i32 = 15;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ConfigurationAcknowledged;

    /// ContainerButtonClick (ID: 16)
    pub const CONTAINER_BUTTON_CLICK_ID: i32 = 16;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerButtonClick;

    /// ContainerClick (ID: 17)
    pub const CONTAINER_CLICK_ID: i32 = 17;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerClick;

    /// ContainerClose (ID: 18)
    pub const CONTAINER_CLOSE_ID: i32 = 18;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerClose;

    /// ContainerSlotStateChanged (ID: 19)
    pub const CONTAINER_SLOT_STATE_CHANGED_ID: i32 = 19;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ContainerSlotStateChanged;

    /// CookieResponse (ID: 20)
    pub const COOKIE_RESPONSE_ID: i32 = 20;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CookieResponse;

    /// CustomPayload (ID: 21)
    pub const CUSTOM_PAYLOAD_ID: i32 = 21;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomPayload;

    /// DebugSubscriptionRequest (ID: 22)
    pub const DEBUG_SUBSCRIPTION_REQUEST_ID: i32 = 22;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct DebugSubscriptionRequest;

    /// EditBook (ID: 23)
    pub const EDIT_BOOK_ID: i32 = 23;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct EditBook;

    /// EntityTagQuery (ID: 24)
    pub const ENTITY_TAG_QUERY_ID: i32 = 24;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct EntityTagQuery;

    /// Interact (ID: 25)
    pub const INTERACT_ID: i32 = 25;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Interact;

    /// JigsawGenerate (ID: 26)
    pub const JIGSAW_GENERATE_ID: i32 = 26;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct JigsawGenerate;

    /// KeepAlive (ID: 27)
    pub const KEEP_ALIVE_ID: i32 = 27;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct KeepAlive;

    /// LockDifficulty (ID: 28)
    pub const LOCK_DIFFICULTY_ID: i32 = 28;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct LockDifficulty;

    /// MovePlayerPos (ID: 29)
    pub const MOVE_PLAYER_POS_ID: i32 = 29;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MovePlayerPos;

    /// MovePlayerPosRot (ID: 30)
    pub const MOVE_PLAYER_POS_ROT_ID: i32 = 30;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MovePlayerPosRot;

    /// MovePlayerRot (ID: 31)
    pub const MOVE_PLAYER_ROT_ID: i32 = 31;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MovePlayerRot;

    /// MovePlayerStatusOnly (ID: 32)
    pub const MOVE_PLAYER_STATUS_ONLY_ID: i32 = 32;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MovePlayerStatusOnly;

    /// MoveVehicle (ID: 33)
    pub const MOVE_VEHICLE_ID: i32 = 33;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct MoveVehicle;

    /// PaddleBoat (ID: 34)
    pub const PADDLE_BOAT_ID: i32 = 34;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PaddleBoat;

    /// PickItemFromBlock (ID: 35)
    pub const PICK_ITEM_FROM_BLOCK_ID: i32 = 35;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PickItemFromBlock;

    /// PickItemFromEntity (ID: 36)
    pub const PICK_ITEM_FROM_ENTITY_ID: i32 = 36;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PickItemFromEntity;

    /// PingRequest (ID: 37)
    pub const PING_REQUEST_ID: i32 = 37;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PingRequest;

    /// PlaceRecipe (ID: 38)
    pub const PLACE_RECIPE_ID: i32 = 38;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlaceRecipe;

    /// PlayerAbilities (ID: 39)
    pub const PLAYER_ABILITIES_ID: i32 = 39;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerAbilities;

    /// PlayerAction (ID: 40)
    pub const PLAYER_ACTION_ID: i32 = 40;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerAction;

    /// PlayerCommand (ID: 41)
    pub const PLAYER_COMMAND_ID: i32 = 41;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerCommand;

    /// PlayerInput (ID: 42)
    pub const PLAYER_INPUT_ID: i32 = 42;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerInput;

    /// PlayerLoaded (ID: 43)
    pub const PLAYER_LOADED_ID: i32 = 43;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct PlayerLoaded;

    /// Pong (ID: 44)
    pub const PONG_ID: i32 = 44;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Pong;

    /// RecipeBookChangeSettings (ID: 45)
    pub const RECIPE_BOOK_CHANGE_SETTINGS_ID: i32 = 45;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookChangeSettings;

    /// RecipeBookSeenRecipe (ID: 46)
    pub const RECIPE_BOOK_SEEN_RECIPE_ID: i32 = 46;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RecipeBookSeenRecipe;

    /// RenameItem (ID: 47)
    pub const RENAME_ITEM_ID: i32 = 47;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct RenameItem;

    /// ResourcePack (ID: 48)
    pub const RESOURCE_PACK_ID: i32 = 48;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct ResourcePack;

    /// SeenAdvancements (ID: 49)
    pub const SEEN_ADVANCEMENTS_ID: i32 = 49;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SeenAdvancements;

    /// SelectTrade (ID: 50)
    pub const SELECT_TRADE_ID: i32 = 50;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SelectTrade;

    /// SetBeacon (ID: 51)
    pub const SET_BEACON_ID: i32 = 51;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetBeacon;

    /// SetCarriedItem (ID: 52)
    pub const SET_CARRIED_ITEM_ID: i32 = 52;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCarriedItem;

    /// SetCommandBlock (ID: 53)
    pub const SET_COMMAND_BLOCK_ID: i32 = 53;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCommandBlock;

    /// SetCommandMinecart (ID: 54)
    pub const SET_COMMAND_MINECART_ID: i32 = 54;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCommandMinecart;

    /// SetCreativeModeSlot (ID: 55)
    pub const SET_CREATIVE_MODE_SLOT_ID: i32 = 55;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetCreativeModeSlot;

    /// SetJigsawBlock (ID: 56)
    pub const SET_JIGSAW_BLOCK_ID: i32 = 56;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetJigsawBlock;

    /// SetStructureBlock (ID: 57)
    pub const SET_STRUCTURE_BLOCK_ID: i32 = 57;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetStructureBlock;

    /// SetTestBlock (ID: 58)
    pub const SET_TEST_BLOCK_ID: i32 = 58;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SetTestBlock;

    /// SignUpdate (ID: 59)
    pub const SIGN_UPDATE_ID: i32 = 59;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct SignUpdate;

    /// Swing (ID: 60)
    pub const SWING_ID: i32 = 60;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct Swing;

    /// TeleportToEntity (ID: 61)
    pub const TELEPORT_TO_ENTITY_ID: i32 = 61;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TeleportToEntity;

    /// TestInstanceBlockAction (ID: 62)
    pub const TEST_INSTANCE_BLOCK_ACTION_ID: i32 = 62;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct TestInstanceBlockAction;

    /// UseItemOn (ID: 63)
    pub const USE_ITEM_ON_ID: i32 = 63;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UseItemOn;

    /// UseItem (ID: 64)
    pub const USE_ITEM_ID: i32 = 64;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct UseItem;

    /// CustomClickAction (ID: 65)
    pub const CUSTOM_CLICK_ACTION_ID: i32 = 65;

    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
    pub struct CustomClickAction;

}
