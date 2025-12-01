//! Spatial cells with RGB coloring.

/// RGB color for spatial partitioning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}

impl Color {
    /// Get all colors in execution order.
    pub const ALL: [Color; 3] = [Color::Red, Color::Green, Color::Blue];

    /// Get the next color in the cycle.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Red => Self::Green,
            Self::Green => Self::Blue,
            Self::Blue => Self::Red,
        }
    }
}

/// Unique identifier for a spatial cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellId(pub u32);

/// A spatial cell containing entities.
#[derive(Debug)]
pub struct Cell {
    /// Cell identifier.
    pub id: CellId,
    /// RGB color for parallel scheduling.
    pub color: Color,
    /// Grid coordinates.
    pub x: i32,
    pub y: i32,
}

impl Cell {
    /// Create a new cell.
    #[must_use]
    pub const fn new(id: CellId, color: Color, x: i32, y: i32) -> Self {
        Self { id, color, x, y }
    }
}
