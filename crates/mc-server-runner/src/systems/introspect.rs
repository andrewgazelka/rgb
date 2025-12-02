//! Introspect system for processing dashboard requests.
//!
//! This system runs every tick and processes any pending introspection
//! requests from the web dashboard.

use rgb_ecs::{Entity, World};
use rgb_ecs_introspect::{
    ChunksResponse, ComponentResponse, ComponentTypesResponse, EntityResponse, HistoryResponse,
    IntrospectIngress, IntrospectRequest, ListEntitiesResponse, QueryResponse, SpawnResponse,
    UpdateResponse, WorldResponse,
    protocol::{EntitySummary, QueryResultRow},
};

use crate::components::ChunkPos;

/// Process all pending introspection requests.
pub fn system_process_introspect(world: &mut World) {
    let Some(ingress) = world.get::<IntrospectIngress>(Entity::WORLD) else {
        return;
    };

    let rx = ingress.rx.clone();
    let registry = ingress.registry.clone();

    // Process all pending requests (non-blocking)
    while let Ok(request) = rx.try_recv() {
        match request {
            IntrospectRequest::GetWorld { response } => {
                let entity_count = world.entity_count();
                let archetype_count = world.archetype_count();
                let component_count = registry.len();

                let _ = response.send(WorldResponse {
                    entity_count,
                    archetype_count,
                    component_count,
                    globals: serde_json::json!({}),
                });
            }

            IntrospectRequest::ListEntities {
                filter: _,
                limit,
                offset,
                response,
            } => {
                let mut entities = Vec::new();
                let offset = offset.unwrap_or(0);
                let limit = limit.unwrap_or(100);

                // Iterate all entities
                for entity in world.entities_iter().skip(offset).take(limit) {
                    // Try Name component first, fall back to world entity name
                    let name = world
                        .get::<crate::components::Name>(entity)
                        .map(|n| n.value.clone())
                        .or_else(|| {
                            world
                                .entity_name(entity)
                                .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
                        });

                    let components: Vec<String> = registry
                        .iter()
                        .filter_map(|info| {
                            if world.has_by_id(entity, info.id()) {
                                Some(info.name.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();

                    entities.push(EntitySummary {
                        id: entity.to_bits(),
                        name,
                        components,
                    });
                }

                let total = world.entity_count() as usize;
                let _ = response.send(ListEntitiesResponse { entities, total });
            }

            IntrospectRequest::GetEntity { entity, response } => {
                if !world.is_alive(entity) {
                    let _ = response.send(EntityResponse {
                        found: false,
                        id: entity.to_bits(),
                        name: None,
                        components: vec![],
                        parent: None,
                        children: vec![],
                    });
                    continue;
                }

                // Try Name component first, fall back to world entity name
                let name = world
                    .get::<crate::components::Name>(entity)
                    .map(|n| n.value.clone())
                    .or_else(|| {
                        world
                            .entity_name(entity)
                            .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
                    });

                let mut components = Vec::new();
                for info in registry.iter() {
                    if let Some(value) = info.get_json(world, entity) {
                        components.push(rgb_ecs_introspect::protocol::ComponentValue {
                            name: info.name.to_string(),
                            full_name: info.full_name.to_string(),
                            value,
                            is_opaque: info.is_opaque,
                            schema: None,
                        });
                    }
                }

                let _ = response.send(EntityResponse {
                    found: true,
                    id: entity.to_bits(),
                    name,
                    components,
                    parent: None,
                    children: vec![],
                });
            }

            IntrospectRequest::GetComponent {
                entity,
                component,
                response,
            } => {
                let value = registry.get_by_name(&component).and_then(|info| {
                    info.get_json(world, entity).map(|v| {
                        rgb_ecs_introspect::protocol::ComponentValue {
                            name: info.name.to_string(),
                            full_name: info.full_name.to_string(),
                            value: v,
                            is_opaque: info.is_opaque,
                            schema: None,
                        }
                    })
                });

                let _ = response.send(ComponentResponse {
                    found: value.is_some(),
                    value,
                });
            }

            IntrospectRequest::UpdateComponent {
                entity,
                component,
                value,
                response,
            } => {
                let result = registry
                    .get_by_name(&component)
                    .ok_or_else(|| format!("Component not found: {component}"))
                    .and_then(|info| {
                        info.set_json(world, entity, &value)
                            .map_err(|e| e.to_string())
                    });

                let _ = response.send(UpdateResponse {
                    success: result.is_ok(),
                    error: result.err(),
                });
            }

            IntrospectRequest::AddComponent {
                entity,
                component,
                value,
                response,
            } => {
                let result = registry
                    .get_by_name(&component)
                    .ok_or_else(|| format!("Component not found: {component}"))
                    .and_then(|info| {
                        info.set_json(world, entity, &value)
                            .map_err(|e| e.to_string())
                    });

                let _ = response.send(UpdateResponse {
                    success: result.is_ok(),
                    error: result.err(),
                });
            }

            IntrospectRequest::RemoveComponent {
                entity,
                component,
                response,
            } => {
                let result = registry
                    .get_by_name(&component)
                    .ok_or_else(|| format!("Component not found: {component}"))
                    .map(|info| {
                        world.remove_by_id(entity, info.id());
                    });

                let _ = response.send(UpdateResponse {
                    success: result.is_ok(),
                    error: result.err(),
                });
            }

            IntrospectRequest::SpawnEntity {
                name,
                components: _,
                response,
            } => {
                let entity = world.spawn(());
                if let Some(name_str) = name {
                    world.insert(entity, crate::components::Name { value: name_str });
                }
                let _ = response.send(SpawnResponse {
                    success: true,
                    entity: Some(entity.to_bits()),
                    error: None,
                });
            }

            IntrospectRequest::DespawnEntity { entity, response } => {
                if world.is_alive(entity) {
                    world.despawn(entity);
                    let _ = response.send(UpdateResponse {
                        success: true,
                        error: None,
                    });
                } else {
                    let _ = response.send(UpdateResponse {
                        success: false,
                        error: Some("Entity not found".into()),
                    });
                }
            }

            IntrospectRequest::Query { spec, response } => {
                let start = std::time::Instant::now();
                let mut entities = Vec::new();
                let offset = spec.offset.unwrap_or(0);
                let limit = spec.limit.unwrap_or(100);

                // Get required component IDs
                let required_ids: Vec<_> = spec
                    .with
                    .iter()
                    .filter_map(|name| registry.get_by_name(name).map(|i| i.id()))
                    .collect();

                for entity in world.entities_iter().skip(offset).take(limit) {
                    // Check if entity has all required components
                    let has_all = required_ids.iter().all(|id| world.has_by_id(entity, *id));

                    if !has_all {
                        continue;
                    }

                    // Try Name component first, fall back to world entity name
                    let name = world
                        .get::<crate::components::Name>(entity)
                        .map(|n| n.value.clone())
                        .or_else(|| {
                            world
                                .entity_name(entity)
                                .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
                        });

                    let mut components = serde_json::Map::new();
                    for comp_name in &spec.with {
                        if let Some(info) = registry.get_by_name(comp_name) {
                            if let Some(value) = info.get_json(world, entity) {
                                components.insert(comp_name.clone(), value);
                            }
                        }
                    }

                    entities.push(QueryResultRow {
                        entity: entity.to_bits(),
                        name,
                        components,
                    });
                }

                let execution_time_us = start.elapsed().as_micros() as u64;
                let total = entities.len();

                let _ = response.send(QueryResponse {
                    entities,
                    total,
                    execution_time_us,
                });
            }

            IntrospectRequest::GetComponentTypes { response } => {
                let types = registry
                    .iter()
                    .map(|info| rgb_ecs_introspect::protocol::ComponentTypeInfo {
                        id: info.id().as_raw(),
                        name: info.name.to_string(),
                        full_name: info.full_name.to_string(),
                        size: info.size(),
                        is_opaque: info.is_opaque,
                        schema: None,
                    })
                    .collect();

                let _ = response.send(ComponentTypesResponse { types });
            }

            IntrospectRequest::GetChunks { response } => {
                let mut chunks = Vec::new();

                for entity in world.entities_iter() {
                    if let Some(chunk_pos) = world.get::<ChunkPos>(entity) {
                        chunks.push(rgb_ecs_introspect::protocol::ChunkInfo {
                            x: chunk_pos.x,
                            z: chunk_pos.z,
                            color: "green".to_string(),
                            loaded: true,
                        });
                    }
                }

                let _ = response.send(ChunksResponse { chunks });
            }

            IntrospectRequest::GetHistory {
                entity: _,
                component: _,
                limit: _,
                response,
            } => {
                // TODO: Implement history retrieval
                let _ = response.send(HistoryResponse {
                    entries: vec![],
                    total: 0,
                });
            }

            IntrospectRequest::RevertToEntry {
                entry_id: _,
                response,
            } => {
                // TODO: Implement revert
                let _ = response.send(UpdateResponse {
                    success: false,
                    error: Some("Not implemented".into()),
                });
            }
        }
    }
}
