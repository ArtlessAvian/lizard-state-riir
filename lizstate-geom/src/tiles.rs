/// Each square tile has four neighbors.
/// A tile's up neighbor's down neighbor must be itself.
/// A tile's right neighbor's left neighbor must be itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub const fn inverse(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

/// Subgraphs of the grid graph.
///
/// Usually, symmetry makes it easier to implement `go` than the individual directions.
///
/// Implementors must ensure:
/// `t.up() == None || t.up().down() == Some(t)`
/// `t.right() == None || t.right().left() == Some(t)`
///
/// An example of a space that does not work is a
///
pub trait Grid: Sized {
    type NeighborType;

    #[must_use]
    fn go(&self, dir: Direction) -> Option<Self::NeighborType>;

    #[must_use]
    fn up(&self) -> Option<Self::NeighborType> {
        self.go(Direction::Up)
    }
    #[must_use]
    fn down(&self) -> Option<Self::NeighborType> {
        self.go(Direction::Down)
    }
    #[must_use]
    fn left(&self) -> Option<Self::NeighborType> {
        self.go(Direction::Left)
    }
    #[must_use]
    fn right(&self) -> Option<Self::NeighborType> {
        self.go(Direction::Right)
    }
}

/// A coordinate for each tile.
///
/// A tile's up neighbor must have coordinate with y + 1 of its own.
/// A tile's right neighbor must have coordinate with x + 1 of its own.
///
/// Do not implement this if this is not possible.
pub trait PlanarProjection: Grid {
    fn project_coords(&self) -> (i32, i32);
}

/// A convenience to avoid `and_then` all over the place.
///
/// TECHNICALLY meets the Grid implementor stuff.
impl<T: Grid> Grid for Option<T> {
    type NeighborType = T::NeighborType;

    fn go(&self, dir: Direction) -> Option<Self::NeighborType> {
        self.as_ref()?.go(dir)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

impl Tile {
    // Prefer over Default::Default.
    pub const ZERO: Tile = Tile { x: 0, y: 0 };

    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Grid for Tile {
    type NeighborType = Tile;

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
