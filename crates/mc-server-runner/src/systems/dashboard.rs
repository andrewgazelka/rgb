//! Dashboard system for processing introspection requests.
//!
//! This system runs every tick and processes any pending dashboard
//! requests from the web server.

use std::collections::HashMap;
use std::time::Instant;

use flecs_ecs::prelude::*;
use flecs_history::HistoryTracker;

use crate::components::{
    ChunkPos, Connection, ConnectionId, EntityId, GameMode, Player, Position, ProtocolState,
    Rotation, Uuid,
};
use crate::dashboard::{
    ChunkInfo, ComponentValue, DashboardChannels, DashboardRequest, EntityDetails, EntitySummary,
    HistoryEntryInfo, HistoryResponse, ListEntitiesResponse, PlayerInfo, PositionInfo,
    QueryResponse, QueryResultRow, WorldInfo,
};

/// Get entity name, returning None if empty.
fn get_entity_name(entity: &EntityView<'_>) -> Option<String> {
    let name = entity.name();
    if name.is_empty() { None } else { Some(name) }
}

/// Build a query that returns user entities, filtering out internal Flecs entities.
#[rustfmt::skip]
fn user_entities_query(world: &World) -> Query<()> {
    world.query::<()>()
        .with(flecs::Wildcard::id())
        .without((flecs::ChildOf::ID, flecs::Flecs::ID)).self_().up()
        .without(flecs::Module::id()).self_().up()
        .with(flecs::Prefab::id()).optional()
        .with(flecs::Disabled::id()).optional()
        .build()
}

/// Helper to create a ComponentValue with full details.
fn make_component_value(name: &str, value: serde_json::Value) -> ComponentValue {
    ComponentValue {
        name: name.to_string(),
        full_name: name.to_string(),
        value,
        is_opaque: false,
        schema: None,
    }
}

/// Get all components for an entity as ComponentValue list.
fn get_entity_components(entity: &EntityView<'_>) -> Vec<ComponentValue> {
    let mut components = Vec::new();

    if let Some(pos) = entity.try_get::<&Position>(|p| *p) {
        components.push(make_component_value(
            "Position",
            serde_json::json!({"x": pos.x, "y": pos.y, "z": pos.z}),
        ));
    }

    if let Some(rot) = entity.try_get::<&Rotation>(|r| *r) {
        components.push(make_component_value(
            "Rotation",
            serde_json::json!({"yaw": rot.yaw, "pitch": rot.pitch}),
        ));
    }

    if let Some(uuid) = entity.try_get::<&Uuid>(|u| u.0) {
        components.push(make_component_value(
            "Uuid",
            serde_json::json!(format!("{:032x}", uuid)),
        ));
    }

    if let Some(entity_id) = entity.try_get::<&EntityId>(|e| e.value) {
        components.push(make_component_value(
            "EntityId",
            serde_json::json!(entity_id),
        ));
    }

    if let Some(game_mode) = entity.try_get::<&GameMode>(|g| *g) {
        components.push(make_component_value(
            "GameMode",
            serde_json::json!(format!("{:?}", game_mode)),
        ));
    }

    if let Some(conn_id) = entity.try_get::<&ConnectionId>(|c| c.0) {
        components.push(make_component_value(
            "ConnectionId",
            serde_json::json!(conn_id),
        ));
    }

    if let Some(state) = entity.try_get::<&ProtocolState>(|s| *s) {
        components.push(make_component_value(
            "ProtocolState",
            serde_json::json!(format!("{:?}", state.0)),
        ));
    }

    if let Some(chunk_pos) = entity.try_get::<&ChunkPos>(|c| *c) {
        components.push(make_component_value(
            "ChunkPos",
            serde_json::json!({"x": chunk_pos.x, "z": chunk_pos.z}),
        ));
    }

    // Tag components
    if entity.has(Player) {
        components.push(make_component_value("Player", serde_json::json!(true)));
    }

    if entity.has(Connection) {
        components.push(make_component_value("Connection", serde_json::json!(true)));
    }

    components
}

/// Get components as a HashMap for query results.
fn get_entity_components_map(entity: &EntityView<'_>) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();

    if let Some(pos) = entity.try_get::<&Position>(|p| *p) {
        map.insert(
            "Position".to_string(),
            serde_json::json!({"x": pos.x, "y": pos.y, "z": pos.z}),
        );
    }

    if let Some(rot) = entity.try_get::<&Rotation>(|r| *r) {
        map.insert(
            "Rotation".to_string(),
            serde_json::json!({"yaw": rot.yaw, "pitch": rot.pitch}),
        );
    }

    if let Some(uuid) = entity.try_get::<&Uuid>(|u| u.0) {
        map.insert(
            "Uuid".to_string(),
            serde_json::json!(format!("{:032x}", uuid)),
        );
    }

    if let Some(entity_id) = entity.try_get::<&EntityId>(|e| e.value) {
        map.insert("EntityId".to_string(), serde_json::json!(entity_id));
    }

    if let Some(game_mode) = entity.try_get::<&GameMode>(|g| *g) {
        map.insert(
            "GameMode".to_string(),
            serde_json::json!(format!("{:?}", game_mode)),
        );
    }

    if let Some(conn_id) = entity.try_get::<&ConnectionId>(|c| c.0) {
        map.insert("ConnectionId".to_string(), serde_json::json!(conn_id));
    }

    if let Some(state) = entity.try_get::<&ProtocolState>(|s| *s) {
        map.insert(
            "ProtocolState".to_string(),
            serde_json::json!(format!("{:?}", state.0)),
        );
    }

    if let Some(chunk_pos) = entity.try_get::<&ChunkPos>(|c| *c) {
        map.insert(
            "ChunkPos".to_string(),
            serde_json::json!({"x": chunk_pos.x, "z": chunk_pos.z}),
        );
    }

    if entity.has(Player) {
        map.insert("Player".to_string(), serde_json::json!(true));
    }

    if entity.has(Connection) {
        map.insert("Connection".to_string(), serde_json::json!(true));
    }

    map
}

/// Compute chunk color based on region coordinates.
fn chunk_color(x: i32, z: i32) -> &'static str {
    // Region is 32x32 chunks
    let rx = x.div_euclid(32);
    let rz = z.div_euclid(32);
    // Simple 3-coloring based on region
    match (rx + rz).rem_euclid(3) {
        0 => "red",
        1 => "green",
        _ => "blue",
    }
}

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
                let count = user_entities_query(world).count() as usize;
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
                let query = user_entities_query(world);
                let total = query.count() as usize;

                let mut entities = Vec::new();
                let mut skipped = 0;

                query.each_entity(|entity, _| {
                    if skipped < offset {
                        skipped += 1;
                        return;
                    }
                    if entities.len() >= limit {
                        return;
                    }

                    let name = get_entity_name(&entity);
                    let comps = get_entity_components(&entity);
                    let component_names: Vec<String> =
                        comps.iter().map(|c| c.name.clone()).collect();

                    entities.push(EntitySummary {
                        id: entity.id().0,
                        name,
                        components: component_names,
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

                let name = get_entity_name(&entity);
                let components = get_entity_components(&entity);

                // Get parent if entity has ChildOf relationship
                let parent = entity.target(flecs::ChildOf::ID, 0).map(|p| p.id().0);

                // Get children
                let mut children = Vec::new();
                // Query for entities that have (ChildOf, this_entity)
                world
                    .query::<()>()
                    .with((flecs::ChildOf::ID, entity.id()))
                    .build()
                    .each_entity(|child, _| {
                        children.push(child.id().0);
                    });

                let _ = response.send(Some(EntityDetails {
                    found: true,
                    id,
                    name,
                    components,
                    parent,
                    children,
                }));
            }

            DashboardRequest::ListPlayers { response } => {
                let mut players = Vec::new();

                world
                    .query::<()>()
                    .with(Player)
                    .build()
                    .each_entity(|entity, _| {
                        let name = get_entity_name(&entity);
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
                        color: chunk_color(chunk_pos.x, chunk_pos.z).to_string(),
                        loaded: true,
                    });
                });

                let _ = response.send(chunks);
            }

            DashboardRequest::GetEntityHistory {
                id,
                limit,
                response,
            } => {
                let entity = Entity::new(id);
                let entries = history.get_entity_history(world, entity);
                let total = entries.len();

                // Convert to the expected format
                let history_entries: Vec<HistoryEntryInfo> = entries
                    .into_iter()
                    .take(limit)
                    .enumerate()
                    .map(|(idx, e)| {
                        // Try to deserialize the data if possible
                        let new_value = if e.data.is_empty() {
                            None
                        } else {
                            // For now, just show data size since we don't have the deserialize info
                            Some(serde_json::json!({"_raw_size": e.data.len()}))
                        };

                        HistoryEntryInfo {
                            id: idx as u64,
                            timestamp: e.tick * 50, // Approximate: 50ms per tick
                            entity: id,
                            component: format!("component_{}", e.component_id),
                            old_value: None,
                            new_value,
                            source: "system".to_string(),
                        }
                    })
                    .collect();

                let _ = response.send(HistoryResponse {
                    entries: history_entries,
                    total,
                });
            }

            DashboardRequest::Query { spec, response } => {
                let start = Instant::now();
                let limit = spec.limit.unwrap_or(100);
                let offset = spec.offset.unwrap_or(0);

                let mut entities = Vec::new();
                let mut total = 0;
                let mut skipped = 0;

                // For now, we support filtering by "Position" component for the map
                let has_position_filter = spec.with.iter().any(|c| c == "Position");

                if has_position_filter {
                    world
                        .query::<&Position>()
                        .build()
                        .each_entity(|entity, pos| {
                            total += 1;

                            if skipped < offset {
                                skipped += 1;
                                return;
                            }
                            if entities.len() >= limit {
                                return;
                            }

                            let name = get_entity_name(&entity);
                            let mut components = HashMap::new();
                            components.insert(
                                "Position".to_string(),
                                serde_json::json!({"x": pos.x, "y": pos.y, "z": pos.z}),
                            );

                            // Add other requested components
                            let all_comps = get_entity_components_map(&entity);
                            for comp_name in &spec.with {
                                if let Some(val) = all_comps.get(comp_name) {
                                    components.insert(comp_name.clone(), val.clone());
                                }
                            }
                            for comp_name in &spec.optional {
                                if let Some(val) = all_comps.get(comp_name) {
                                    components.insert(comp_name.clone(), val.clone());
                                }
                            }

                            entities.push(QueryResultRow {
                                entity: entity.id().0,
                                name,
                                components,
                            });
                        });
                } else {
                    // Generic query - return all user entities
                    user_entities_query(world).each_entity(|entity, _| {
                        total += 1;

                        if skipped < offset {
                            skipped += 1;
                            return;
                        }
                        if entities.len() >= limit {
                            return;
                        }

                        let name = get_entity_name(&entity);
                        let all_comps = get_entity_components_map(&entity);

                        // Filter by required components
                        let has_all_required = spec.with.iter().all(|c| all_comps.contains_key(c));
                        if !has_all_required {
                            return;
                        }

                        // Filter by without
                        let has_excluded = spec.without.iter().any(|c| all_comps.contains_key(c));
                        if has_excluded {
                            return;
                        }

                        let mut components = HashMap::new();
                        for comp_name in &spec.with {
                            if let Some(val) = all_comps.get(comp_name) {
                                components.insert(comp_name.clone(), val.clone());
                            }
                        }
                        for comp_name in &spec.optional {
                            if let Some(val) = all_comps.get(comp_name) {
                                components.insert(comp_name.clone(), val.clone());
                            }
                        }

                        // If no specific components requested, return all
                        if spec.with.is_empty() && spec.optional.is_empty() {
                            components = all_comps;
                        }

                        entities.push(QueryResultRow {
                            entity: entity.id().0,
                            name,
                            components,
                        });
                    });
                }

                let execution_time_us = start.elapsed().as_micros() as u64;

                let _ = response.send(QueryResponse {
                    entities,
                    total,
                    execution_time_us,
                });
            }
        }
    }
}
