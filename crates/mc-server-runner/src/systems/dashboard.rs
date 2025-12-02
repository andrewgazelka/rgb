//! Dashboard system for processing introspection requests.
//!
//! This system runs every tick and processes any pending dashboard
//! requests from the web server.

use flecs_ecs::prelude::*;
use flecs_history::HistoryTracker;

use crate::components::{
    ChunkPos, Connection, ConnectionId, EntityId, GameMode, Name, Player, Position, ProtocolState,
    Rotation, Uuid,
};
use crate::dashboard::{
    ChunkInfo, ComponentValue, DashboardChannels, DashboardRequest, EntityDetails, EntitySummary,
    HistoryEntryInfo, ListEntitiesResponse, PlayerInfo, PositionInfo, WorldInfo,
};

/// Process all pending dashboard requests.
pub fn system_process_dashboard(
    world: &World,
    channels: &DashboardChannels,
    history: &HistoryTracker,
) {
    // Process all pending requests (non-blocking)
    while let Ok(request) = channels.request_rx.try_recv() {
        match request {
            DashboardRequest::GetWorld { response } => {
                let mut count = 0;
                world
                    .query::<()>()
                    .with(flecs::Wildcard::id())
                    .without((flecs::ChildOf::ID, flecs::Flecs::ID))
                    .self_()
                    .up()
                    .without(flecs::Module::id())
                    .self_()
                    .up()
                    .with(flecs::Prefab::id())
                    .optional()
                    .with(flecs::Disabled::id())
                    .optional()
                    .build()
                    .each(|_| {
                        count += 1;
                    });
                let _ = response.send(WorldInfo {
                    entity_count: count,
                    archetype_count: 0,
                    component_count: 0,
                    globals: serde_json::json!({}),
                });
            }

            DashboardRequest::ListEntities {
                limit,
                offset,
                response,
            } => {
                let mut entities = Vec::new();
                let mut total = 0;
                let mut skipped = 0;

                world
                    .query::<()>()
                    .with(flecs::Wildcard::id())
                    .without((flecs::ChildOf::ID, flecs::Flecs::ID))
                    .self_()
                    .up()
                    .without(flecs::Module::id())
                    .self_()
                    .up()
                    .with(flecs::Prefab::id())
                    .optional()
                    .with(flecs::Disabled::id())
                    .optional()
                    .build()
                    .each_entity(|entity, _| {
                        total += 1;

                        if skipped < offset {
                            skipped += 1;
                            return;
                        }
                        if entities.len() >= limit {
                            return;
                        }

                        let name = entity.try_get::<&Name>(|n| n.value.clone());

                        let mut components = Vec::new();
                        if entity.has(Player) {
                            components.push("Player".to_string());
                        }
                        if entity.has(Connection) {
                            components.push("Connection".to_string());
                        }
                        if entity.try_get::<&Position>(|_| ()).is_some() {
                            components.push("Position".to_string());
                        }
                        if entity.try_get::<&ChunkPos>(|_| ()).is_some() {
                            components.push("ChunkPos".to_string());
                        }

                        entities.push(EntitySummary {
                            id: entity.id().0,
                            name,
                            components,
                        });
                    });

                let _ = response.send(ListEntitiesResponse { entities, total });
            }

            DashboardRequest::GetEntity { id, response } => {
                let entity = world.entity_from_id(Entity::new(id));

                if !entity.is_alive() {
                    let _ = response.send(None);
                    continue;
                }

                let name = entity.try_get::<&Name>(|n| n.value.clone());
                let mut components = Vec::new();

                // Try to get each known component and serialize it
                if let Some(pos) = entity.try_get::<&Position>(|p| *p) {
                    components.push(ComponentValue {
                        name: "Position".to_string(),
                        value: serde_json::json!({
                            "x": pos.x,
                            "y": pos.y,
                            "z": pos.z,
                        }),
                    });
                }

                if let Some(rot) = entity.try_get::<&Rotation>(|r| *r) {
                    components.push(ComponentValue {
                        name: "Rotation".to_string(),
                        value: serde_json::json!({
                            "yaw": rot.yaw,
                            "pitch": rot.pitch,
                        }),
                    });
                }

                if let Some(uuid) = entity.try_get::<&Uuid>(|u| u.0) {
                    components.push(ComponentValue {
                        name: "Uuid".to_string(),
                        value: serde_json::json!(format!("{:032x}", uuid)),
                    });
                }

                if let Some(entity_id) = entity.try_get::<&EntityId>(|e| e.value) {
                    components.push(ComponentValue {
                        name: "EntityId".to_string(),
                        value: serde_json::json!(entity_id),
                    });
                }

                if let Some(game_mode) = entity.try_get::<&GameMode>(|g| *g) {
                    components.push(ComponentValue {
                        name: "GameMode".to_string(),
                        value: serde_json::json!(format!("{:?}", game_mode)),
                    });
                }

                if let Some(conn_id) = entity.try_get::<&ConnectionId>(|c| c.0) {
                    components.push(ComponentValue {
                        name: "ConnectionId".to_string(),
                        value: serde_json::json!(conn_id),
                    });
                }

                if let Some(state) = entity.try_get::<&ProtocolState>(|s| *s) {
                    components.push(ComponentValue {
                        name: "ProtocolState".to_string(),
                        value: serde_json::json!(format!("{:?}", state.0)),
                    });
                }

                if let Some(chunk_pos) = entity.try_get::<&ChunkPos>(|c| *c) {
                    components.push(ComponentValue {
                        name: "ChunkPos".to_string(),
                        value: serde_json::json!({
                            "x": chunk_pos.x,
                            "z": chunk_pos.z,
                        }),
                    });
                }

                // Tag components
                if entity.has(Player) {
                    components.push(ComponentValue {
                        name: "Player".to_string(),
                        value: serde_json::json!(true),
                    });
                }

                if entity.has(Connection) {
                    components.push(ComponentValue {
                        name: "Connection".to_string(),
                        value: serde_json::json!(true),
                    });
                }

                let _ = response.send(Some(EntityDetails {
                    id,
                    name,
                    components,
                }));
            }

            DashboardRequest::ListPlayers { response } => {
                let mut players = Vec::new();

                world
                    .query::<()>()
                    .with(Player)
                    .build()
                    .each_entity(|entity, _| {
                        let name = entity.try_get::<&Name>(|n| n.value.clone());
                        let uuid = entity.try_get::<&Uuid>(|u| format!("{:032x}", u.0));
                        let position = entity.try_get::<&Position>(|p| PositionInfo {
                            x: p.x,
                            y: p.y,
                            z: p.z,
                        });
                        let game_mode = entity.try_get::<&GameMode>(|g| format!("{:?}", g));

                        players.push(PlayerInfo {
                            entity_id: entity.id().0,
                            name,
                            uuid,
                            position,
                            game_mode,
                        });
                    });

                let _ = response.send(players);
            }

            DashboardRequest::ListChunks { response } => {
                let mut chunks = Vec::new();

                world.query::<&ChunkPos>().build().each(|chunk_pos| {
                    chunks.push(ChunkInfo {
                        x: chunk_pos.x,
                        z: chunk_pos.z,
                    });
                });

                let _ = response.send(chunks);
            }

            DashboardRequest::GetEntityHistory { id, response } => {
                let entity = Entity::new(id);
                let entries = history.get_entity_history(world, entity);

                let history_info: Vec<HistoryEntryInfo> = entries
                    .into_iter()
                    .map(|e| HistoryEntryInfo {
                        tick: e.tick,
                        component_id: e.component_id,
                        data_size: e.data.len(),
                    })
                    .collect();

                let _ = response.send(history_info);
            }
        }
    }
}
