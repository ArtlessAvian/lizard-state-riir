use crate::grid::Direction;
use crate::grid::Grid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct Coords {
    pub x: i32,
    pub y: i32,
}

impl Coords {
    // Prefer over Default::Default.
    pub const ZERO: Coords = Coords { x: 0, y: 0 };

    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Grid for Coords {
    type Neighbor = Coords;

    fn go(&self, dir: Direction) -> Option<Self> {
        Some(match dir {
            Direction::Up => Self {
                x: self.x,
                y: self.y.checked_add(1)?,
            },
            Direction::Down => Self {
                x: self.x,
                y: self.y.checked_add(-1)?,
            },
            Direction::Left => Self {
                x: self.x.checked_add(-1)?,
                y: self.y,
            },
            Direction::Right => Self {
                x: self.x.checked_add(1)?,
                y: self.y,
            },
        })
    }
}

/// A coordinate for each tile.
///
/// A tile's up neighbor must have coordinate with y + 1 of its own.
/// A tile's right neighbor must have coordinate with x + 1 of its own.
///
/// Do not implement this if this is not possible.
pub trait PlanarProjection: Grid {
    fn project_coords(&self) -> Coords;
}

impl PlanarProjection for Coords {
    fn project_coords(&self) -> Coords {
        *self
    }
}
