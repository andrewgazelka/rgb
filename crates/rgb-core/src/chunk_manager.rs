use std::collections::HashMap;

use flecs_ecs::prelude::*;

use crate::color::Color;
use crate::components::{
    Active, CellData, Direction, Dirty, NeighborE, NeighborN, NeighborNE, NeighborNW, NeighborS,
    NeighborSE, NeighborSW, NeighborW, NextCellData, SimColor,
};
use crate::pos::ChunkPos;

/// Index for fast chunk lookups by position
#[derive(Component, Default)]
pub struct ChunkIndex {
    pub map: HashMap<ChunkPos, Entity>,
}

/// Spawn a new chunk entity with the given position and cell data
pub fn spawn_chunk(world: &World, pos: ChunkPos, cells: CellData) -> EntityView<'_> {
    let region = pos.containing_region();
    let color = Color::from_region(region);

    let is_active = !cells.is_empty();

    let name = format!("chunk:{}:{}", pos.x, pos.y);
    let chunk = world
        .entity_named(&name)
        .set(pos)
        .set(cells)
        .set(NextCellData::default())
        .set(SimColor(color as u8))
        .add(Dirty);

    if is_active {
        chunk.add(Active);
    }

    chunk
}

/// Link a chunk to its neighbors (bidirectional relationships)
pub fn link_chunk_neighbors(
    world: &World,
    chunk_entity: Entity,
    pos: ChunkPos,
    index: &ChunkIndex,
) {
    let chunk = world.entity_from_id(chunk_entity);

    for dir in Direction::ALL {
        let (dx, dy) = dir.offset();
        let neighbor_pos = pos.offset(dx, dy);

        if let Some(&neighbor_entity) = index.map.get(&neighbor_pos) {
            let neighbor = world.entity_from_id(neighbor_entity);

            // Add relationship from chunk to neighbor
            add_neighbor_relationship(&chunk, neighbor_entity, dir);

            // Add reverse relationship from neighbor to chunk
            add_neighbor_relationship(&neighbor, chunk_entity, dir.opposite());
        }
    }
}

fn add_neighbor_relationship(entity: &EntityView<'_>, target: Entity, dir: Direction) {
    match dir {
        Direction::N => {
            entity.add((NeighborN, target));
        }
        Direction::S => {
            entity.add((NeighborS, target));
        }
        Direction::E => {
            entity.add((NeighborE, target));
        }
        Direction::W => {
            entity.add((NeighborW, target));
        }
        Direction::NE => {
            entity.add((NeighborNE, target));
        }
        Direction::NW => {
            entity.add((NeighborNW, target));
        }
        Direction::SE => {
            entity.add((NeighborSE, target));
        }
        Direction::SW => {
            entity.add((NeighborSW, target));
        }
    }
}

/// Get the neighbor entity in the given direction
pub fn get_neighbor(world: &World, chunk_entity: Entity, dir: Direction) -> Option<Entity> {
    let chunk = world.entity_from_id(chunk_entity);

    let target_view = match dir {
        Direction::N => chunk.target(NeighborN, 0),
        Direction::S => chunk.target(NeighborS, 0),
        Direction::E => chunk.target(NeighborE, 0),
        Direction::W => chunk.target(NeighborW, 0),
        Direction::NE => chunk.target(NeighborNE, 0),
        Direction::NW => chunk.target(NeighborNW, 0),
        Direction::SE => chunk.target(NeighborSE, 0),
        Direction::SW => chunk.target(NeighborSW, 0),
    };

    target_view.map(|v| v.id())
}

/// Unlink a chunk from all its neighbors before removal
pub fn unlink_chunk_neighbors(
    world: &World,
    chunk_entity: Entity,
    pos: ChunkPos,
    index: &ChunkIndex,
) {
    for dir in Direction::ALL {
        let (dx, dy) = dir.offset();
        let neighbor_pos = pos.offset(dx, dy);

        if let Some(&neighbor_entity) = index.map.get(&neighbor_pos) {
            let neighbor = world.entity_from_id(neighbor_entity);

            // Remove reverse relationship from neighbor
            remove_neighbor_relationship(&neighbor, chunk_entity, dir.opposite());
        }
    }
}

fn remove_neighbor_relationship(entity: &EntityView<'_>, target: Entity, dir: Direction) {
    match dir {
        Direction::N => {
            entity.remove((NeighborN, target));
        }
        Direction::S => {
            entity.remove((NeighborS, target));
        }
        Direction::E => {
            entity.remove((NeighborE, target));
        }
        Direction::W => {
            entity.remove((NeighborW, target));
        }
        Direction::NE => {
            entity.remove((NeighborNE, target));
        }
        Direction::NW => {
            entity.remove((NeighborNW, target));
        }
        Direction::SE => {
            entity.remove((NeighborSE, target));
        }
        Direction::SW => {
            entity.remove((NeighborSW, target));
        }
    }
}
