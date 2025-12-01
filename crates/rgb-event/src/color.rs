//! Cell color computation from position.

use rgb_spatial::Color;

/// Cell size in blocks (same as Minecraft chunks).
pub const CELL_SHIFT: u32 = 4; // 16 blocks

/// Compute cell color from world position.
///
/// This is O(1) arithmetic - just a cast, shift, and mod.
/// No need to cache the result.
#[inline]
#[must_use]
pub fn cell_color(x: f64, z: f64) -> Color {
    let cx = (x as i32) >> CELL_SHIFT;
    let cz = (z as i32) >> CELL_SHIFT;
    // (cx + cz) mod 3, handling negatives correctly
    match ((cx + cz) % 3 + 3) % 3 {
        0 => Color::Red,
        1 => Color::Green,
        _ => Color::Blue,
    }
}

/// Compute cell color from integer block coordinates.
#[inline]
#[must_use]
pub fn cell_color_i32(x: i32, z: i32) -> Color {
    let cx = x >> CELL_SHIFT;
    let cz = z >> CELL_SHIFT;
    match ((cx + cz) % 3 + 3) % 3 {
        0 => Color::Red,
        1 => Color::Green,
        _ => Color::Blue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_color_origin() {
        // Cell (0, 0) -> (0 + 0) % 3 = 0 -> Red
        assert_eq!(cell_color(0.0, 0.0), Color::Red);
        assert_eq!(cell_color(15.9, 15.9), Color::Red);
    }

    #[test]
    fn test_cell_color_adjacent() {
        // Cell (1, 0) -> (1 + 0) % 3 = 1 -> Green
        assert_eq!(cell_color(16.0, 0.0), Color::Green);
        // Cell (0, 1) -> (0 + 1) % 3 = 1 -> Green
        assert_eq!(cell_color(0.0, 16.0), Color::Green);
        // Cell (1, 1) -> (1 + 1) % 3 = 2 -> Blue
        assert_eq!(cell_color(16.0, 16.0), Color::Blue);
    }

    #[test]
    fn test_cell_color_negative() {
        // Cell (-1, 0) -> (-1 + 0) % 3 = -1 -> +3 -> 2 -> Blue
        assert_eq!(cell_color(-1.0, 0.0), Color::Blue);
        // Cell (-1, -1) -> (-1 + -1) % 3 = -2 -> +3 -> 1 -> Green
        assert_eq!(cell_color(-1.0, -1.0), Color::Green);
    }

    #[test]
    fn test_no_adjacent_same_color() {
        // Check that no two edge-adjacent cells have the same color
        for cx in -5..5 {
            for cz in -5..5 {
                let x = (cx * 16 + 8) as f64;
                let z = (cz * 16 + 8) as f64;
                let color = cell_color(x, z);

                // Check 4 edge neighbors
                let neighbors = [
                    cell_color(x + 16.0, z),
                    cell_color(x - 16.0, z),
                    cell_color(x, z + 16.0),
                    cell_color(x, z - 16.0),
                ];

                for neighbor_color in neighbors {
                    assert_ne!(
                        color, neighbor_color,
                        "Adjacent cells at ({cx}, {cz}) have same color {color:?}"
                    );
                }
            }
        }
    }
}
