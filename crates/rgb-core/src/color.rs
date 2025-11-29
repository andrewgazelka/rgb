use crate::pos::RegionPos;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Color {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
}

impl Color {
    pub const ALL: [Color; 9] = [
        Color::C0,
        Color::C1,
        Color::C2,
        Color::C3,
        Color::C4,
        Color::C5,
        Color::C6,
        Color::C7,
        Color::C8,
    ];

    pub const fn from_region(pos: RegionPos) -> Self {
        let x = pos.x.rem_euclid(3) as usize;
        let y = pos.y.rem_euclid(3) as usize;
        Self::ALL[y * 3 + x]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_adjacent_same_color() {
        for y in -10..10 {
            for x in -10..10 {
                let pos = RegionPos::new(x, y);
                let color = Color::from_region(pos);

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let neighbor = RegionPos::new(x + dx, y + dy);
                        assert_ne!(
                            color,
                            Color::from_region(neighbor),
                            "adjacency conflict at ({x}, {y}) -> ({}, {})",
                            x + dx,
                            y + dy
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn pattern_visualization() {
        for y in 0..6 {
            for x in 0..9 {
                let c = match Color::from_region(RegionPos::new(x, y)) {
                    Color::C0 => '0',
                    Color::C1 => '1',
                    Color::C2 => '2',
                    Color::C3 => '3',
                    Color::C4 => '4',
                    Color::C5 => '5',
                    Color::C6 => '6',
                    Color::C7 => '7',
                    Color::C8 => '8',
                };
                print!("{c} ");
            }
            println!();
        }
    }
}
