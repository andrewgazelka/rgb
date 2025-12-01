//! Spatial grid with RGB coloring.

use crate::{Cell, CellId, Color};

/// A 2D spatial grid with RGB cell coloring.
pub struct SpatialGrid {
    /// Grid width in cells.
    pub width: u32,
    /// Grid height in cells.
    pub height: u32,
    /// Cell size in world units.
    pub cell_size: f32,
    /// All cells.
    cells: Vec<Cell>,
}

impl SpatialGrid {
    /// Create a new spatial grid.
    #[must_use]
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        let mut cells = Vec::with_capacity((width * height) as usize);

        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let id = CellId((y as u32) * width + (x as u32));
                // RGB brick pattern: offset every other row
                let color = Self::compute_color(x, y);
                cells.push(Cell::new(id, color, x, y));
            }
        }

        Self {
            width,
            height,
            cell_size,
            cells,
        }
    }

    /// Compute RGB color for a cell coordinate.
    ///
    /// Uses a 3-coloring pattern where no two **edge-adjacent** cells (4-connectivity)
    /// share the same color. Note: Diagonal neighbors MAY share colors.
    ///
    /// For 8-connectivity (including diagonals), you would need 4 colors.
    /// The RGB pattern is sufficient when cross-cell interactions only occur
    /// between edge-adjacent cells (e.g., entity movement along axis).
    ///
    /// The pattern shifts each row to ensure vertical neighbors differ:
    /// ```text
    ///   0 1 2 3 4 5
    /// 0 R G B R G B
    /// 1 G B R G B R
    /// 2 B R G B R G
    /// 3 R G B R G B
    /// ```
    ///
    /// For stricter isolation (no diagonal conflicts), use 4 colors with checkerboard variant.
    #[must_use]
    fn compute_color(x: i32, y: i32) -> Color {
        // Shift each row by 1 to ensure vertical neighbors differ
        // (x + y) mod 3 gives proper 4-connectivity coloring
        let index = ((x + y) % 3 + 3) % 3;
        match index {
            0 => Color::Red,
            1 => Color::Green,
            _ => Color::Blue,
        }
    }

    /// Get cells of a specific color for parallel execution.
    pub fn cells_by_color(&self, color: Color) -> impl Iterator<Item = &Cell> {
        self.cells.iter().filter(move |c| c.color == color)
    }

    /// Get a cell by ID.
    #[must_use]
    pub fn get(&self, id: CellId) -> Option<&Cell> {
        self.cells.get(id.0 as usize)
    }

    /// Get cell ID from world coordinates.
    #[must_use]
    pub fn cell_at(&self, world_x: f32, world_y: f32) -> Option<CellId> {
        let x = (world_x / self.cell_size).floor() as i32;
        let y = (world_y / self.cell_size).floor() as i32;

        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            Some(CellId((y as u32) * self.width + (x as u32)))
        } else {
            None
        }
    }

    /// Get total number of cells.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Check if grid is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_coloring_no_edge_adjacent_same_color() {
        let grid = SpatialGrid::new(10, 10, 16.0);

        // Only check 4-connectivity (edge-adjacent), not diagonals
        let edge_neighbors = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        for cell in &grid.cells {
            for &(dx, dy) in &edge_neighbors {
                let nx = cell.x + dx;
                let ny = cell.y + dy;

                if nx >= 0 && nx < 10 && ny >= 0 && ny < 10 {
                    let neighbor_id = CellId((ny as u32) * 10 + (nx as u32));
                    let neighbor = grid.get(neighbor_id).unwrap();

                    assert_ne!(
                        cell.color, neighbor.color,
                        "Edge-adjacent cells ({},{}) and ({},{}) have same color {:?}",
                        cell.x, cell.y, nx, ny, cell.color
                    );
                }
            }
        }
    }

    #[test]
    fn test_cell_at_position() {
        let grid = SpatialGrid::new(10, 10, 16.0);

        assert_eq!(grid.cell_at(0.0, 0.0), Some(CellId(0)));
        assert_eq!(grid.cell_at(15.9, 0.0), Some(CellId(0)));
        assert_eq!(grid.cell_at(16.0, 0.0), Some(CellId(1)));
        assert_eq!(grid.cell_at(0.0, 16.0), Some(CellId(10)));

        // Out of bounds
        assert_eq!(grid.cell_at(-1.0, 0.0), None);
        assert_eq!(grid.cell_at(160.0, 0.0), None);
    }

    #[test]
    fn test_cells_by_color_count() {
        let grid = SpatialGrid::new(9, 9, 16.0);

        let red_count = grid.cells_by_color(Color::Red).count();
        let green_count = grid.cells_by_color(Color::Green).count();
        let blue_count = grid.cells_by_color(Color::Blue).count();

        // With column-based coloring and 9 columns, should be even
        assert_eq!(red_count, 27);
        assert_eq!(green_count, 27);
        assert_eq!(blue_count, 27);
        assert_eq!(red_count + green_count + blue_count, 81);
    }
}
