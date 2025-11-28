use flecs_ecs::prelude::*;
use rgb_core::{
    get_neighbor, link_chunk_neighbors, spawn_chunk, Active, CellData, ChunkIndex, ChunkPos,
    Direction, Dirty, NextCellData, SimColor, CHUNK_SIZE,
};
use std::collections::HashSet;

/// Normalize cell coordinates that may be outside chunk bounds
/// Returns (chunk_pos, local_x, local_y)
pub fn normalize_coords(chunk: ChunkPos, x: i32, y: i32) -> (ChunkPos, usize, usize) {
    let global_x = chunk.x as i64 * CHUNK_SIZE as i64 + x as i64;
    let global_y = chunk.y as i64 * CHUNK_SIZE as i64 + y as i64;

    let new_chunk = ChunkPos::new(
        global_x.div_euclid(CHUNK_SIZE as i64) as i32,
        global_y.div_euclid(CHUNK_SIZE as i64) as i32,
    );

    let local_x = global_x.rem_euclid(CHUNK_SIZE as i64) as usize;
    let local_y = global_y.rem_euclid(CHUNK_SIZE as i64) as usize;

    (new_chunk, local_x, local_y)
}

/// Get a cell value, looking up neighbor chunks if needed
pub fn get_cell(
    world: &World,
    chunk_entity: Entity,
    chunk_pos: ChunkPos,
    chunk_cells: &CellData,
    x: i32,
    y: i32,
) -> bool {
    // Fast path: cell is within current chunk
    if x >= 0 && x < CHUNK_SIZE as i32 && y >= 0 && y < CHUNK_SIZE as i32 {
        return chunk_cells.get(x as usize, y as usize);
    }

    // Slow path: need to look up neighbor chunk
    let (target_chunk_pos, local_x, local_y) = normalize_coords(chunk_pos, x, y);

    // Determine which direction the neighbor is in
    let dx = target_chunk_pos.x - chunk_pos.x;
    let dy = target_chunk_pos.y - chunk_pos.y;

    let dir = match (dx, dy) {
        (0, 1) => Direction::N,
        (0, -1) => Direction::S,
        (1, 0) => Direction::E,
        (-1, 0) => Direction::W,
        (1, 1) => Direction::NE,
        (-1, 1) => Direction::NW,
        (1, -1) => Direction::SE,
        (-1, -1) => Direction::SW,
        _ => return false, // Should not happen for immediate neighbors
    };

    // Get the neighbor entity
    if let Some(neighbor_entity) = get_neighbor(world, chunk_entity, dir) {
        let neighbor = world.entity_from_id(neighbor_entity);
        let mut result = false;
        neighbor.try_get::<&CellData>(|cells| {
            result = cells.get(local_x, local_y);
        });
        result
    } else {
        false // No neighbor = dead cell
    }
}

/// Count live neighbors for a cell
pub fn count_neighbors(
    world: &World,
    chunk_entity: Entity,
    chunk_pos: ChunkPos,
    chunk_cells: &CellData,
    x: i32,
    y: i32,
) -> u8 {
    let mut count = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            if get_cell(world, chunk_entity, chunk_pos, chunk_cells, x + dx, y + dy) {
                count += 1;
            }
        }
    }
    count
}

/// Check if a chunk has activity at its edges that might spawn new cells in neighbors
/// Returns set of directions where new chunks should be created
pub fn check_edge_activity(cells: &CellData) -> HashSet<Direction> {
    let mut needs_neighbors = HashSet::new();
    let max = CHUNK_SIZE - 1;

    // Check north edge (y = max)
    for x in 0..CHUNK_SIZE {
        if cells.get(x, max) {
            needs_neighbors.insert(Direction::N);
            if x == 0 {
                needs_neighbors.insert(Direction::NW);
            }
            if x == max {
                needs_neighbors.insert(Direction::NE);
            }
        }
    }

    // Check south edge (y = 0)
    for x in 0..CHUNK_SIZE {
        if cells.get(x, 0) {
            needs_neighbors.insert(Direction::S);
            if x == 0 {
                needs_neighbors.insert(Direction::SW);
            }
            if x == max {
                needs_neighbors.insert(Direction::SE);
            }
        }
    }

    // Check east edge (x = max)
    for y in 0..CHUNK_SIZE {
        if cells.get(max, y) {
            needs_neighbors.insert(Direction::E);
            if y == 0 {
                needs_neighbors.insert(Direction::SE);
            }
            if y == max {
                needs_neighbors.insert(Direction::NE);
            }
        }
    }

    // Check west edge (x = 0)
    for y in 0..CHUNK_SIZE {
        if cells.get(0, y) {
            needs_neighbors.insert(Direction::W);
            if y == 0 {
                needs_neighbors.insert(Direction::SW);
            }
            if y == max {
                needs_neighbors.insert(Direction::NW);
            }
        }
    }

    needs_neighbors
}

/// Compute the next generation for a chunk
pub fn compute_next_generation(
    world: &World,
    chunk_entity: Entity,
    chunk_pos: ChunkPos,
    cells: &CellData,
) -> [[bool; CHUNK_SIZE]; CHUNK_SIZE] {
    let mut next = [[false; CHUNK_SIZE]; CHUNK_SIZE];

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let alive = cells.get(x, y);
            let neighbors = count_neighbors(world, chunk_entity, chunk_pos, cells, x as i32, y as i32);

            let next_alive = match (alive, neighbors) {
                (true, 2) | (true, 3) => true,
                (false, 3) => true,
                _ => false,
            };

            next[y][x] = next_alive;
        }
    }

    next
}

/// Expand the world by creating empty neighbor chunks where needed
pub fn expand_world(world: &World, index: &mut ChunkIndex) {
    let mut chunks_to_create: HashSet<ChunkPos> = HashSet::new();

    // Collect all positions that need new neighbors
    world.each_entity::<(&ChunkPos, &CellData)>(|entity, (pos, cells)| {
        let needed = check_edge_activity(cells);

        for dir in needed {
            let neighbor_pos = pos.neighbor(dir);

            // Check if neighbor already exists
            if get_neighbor(world, entity.id(), dir).is_none() {
                chunks_to_create.insert(neighbor_pos);
            }
        }
    });

    // Create new chunks
    for pos in chunks_to_create {
        if !index.map.contains_key(&pos) {
            let chunk = spawn_chunk(world, pos, CellData::default());
            // Mark as Active so it gets simulated and can receive cells from neighbors
            chunk.add(Active);
            chunk.add(Dirty);
            index.map.insert(pos, chunk.id());
            link_chunk_neighbors(world, chunk.id(), pos, index);
        }
    }
}

/// Register Game of Life systems with the world
pub fn register_life_systems(world: &World) {
    // System to compute next generation for all active chunks
    // We process by color to ensure no data races (chunks of same color don't share edges)
    world
        .system_named::<(&ChunkPos, &CellData, &mut NextCellData, &SimColor)>("ComputeNextGen")
        .with(Active)
        .each_entity(|e, (pos, cells, next, color)| {
            // We use the color for parallel safety but process all in one system
            // The Flecs scheduler handles the parallelism
            let _ = color; // Color is used for grouping, not logic

            let world = e.world();
            next.cells = compute_next_generation(&world, e.id(), *pos, cells);
        });

    // System to swap buffers (copy next -> current)
    world
        .system_named::<(&mut CellData, &NextCellData)>("SwapBuffers")
        .with(Active)
        .each(|(cells, next)| {
            cells.cells = next.cells;
        });

    // System to mark changed chunks as dirty
    world
        .system_named::<&CellData>("MarkDirty")
        .with(Active)
        .without(Dirty)
        .each_entity(|e, _cells| {
            e.add(Dirty);
        });

    // System to update Active status based on cell count
    world
        .system_named::<&CellData>("UpdateActive")
        .each_entity(|e, cells| {
            if cells.is_empty() {
                e.remove(Active);
            } else if !e.has(Active) {
                e.add(Active);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_world_with_chunk(cells: CellData) -> (World, Entity) {
        let world = World::new();
        let pos = ChunkPos::new(0, 0);
        let chunk = spawn_chunk(&world, pos, cells);
        let entity = chunk.id();

        let mut index = ChunkIndex::default();
        index.map.insert(pos, entity);

        (world, entity)
    }

    #[test]
    fn blinker() {
        let mut cells = CellData::default();
        cells.set(7, 7, true);
        cells.set(8, 7, true);
        cells.set(9, 7, true);

        let (world, entity) = setup_world_with_chunk(cells.clone());
        let pos = ChunkPos::new(0, 0);

        let next = compute_next_generation(&world, entity, pos, &cells);

        assert!(!next[7][7]);
        assert!(next[6][8]);
        assert!(next[7][8]);
        assert!(next[8][8]);
        assert!(!next[7][9]);
    }

    #[test]
    fn block_stable() {
        let mut cells = CellData::default();
        cells.set(0, 0, true);
        cells.set(1, 0, true);
        cells.set(0, 1, true);
        cells.set(1, 1, true);

        let (world, entity) = setup_world_with_chunk(cells.clone());
        let pos = ChunkPos::new(0, 0);

        let next = compute_next_generation(&world, entity, pos, &cells);

        assert!(next[0][0]);
        assert!(next[0][1]);
        assert!(next[1][0]);
        assert!(next[1][1]);
    }
}
