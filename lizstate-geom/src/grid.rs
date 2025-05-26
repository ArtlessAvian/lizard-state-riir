/// Each square tile has four neighbors.
/// A tile's up neighbor's down neighbor must be itself.
/// A tile's right neighbor's left neighbor must be itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
/// Implementors must ensure:
/// `t.up() == None || t.up().down() == Some(t)`
/// `t.right() == None || t.right().left() == Some(t)`
///
/// Examples of bad spaces:
/// * `t.up() == Some(t)` but `t.down() == None`
///
/// We could bound Neighbor: Eq, but it's awkward.
pub trait Grid: Clone {
    type Neighbor;

    #[must_use]
    fn go(&self, dir: Direction) -> Option<Self::Neighbor>;
}

/// A convenience to avoid `and_then` all over the place.
///
/// TECHNICALLY meets the Grid implementor stuff.
impl<T: Grid> Grid for Option<T> {
    type Neighbor = T::Neighbor;

    fn go(&self, dir: Direction) -> Option<Self::Neighbor> {
        self.as_ref()?.go(dir)
    }
}

pub trait GridShortcuts: Grid {
    #[must_use]
    fn up(&self) -> Option<Self::Neighbor> {
        self.go(Direction::Up)
    }
    #[must_use]
    fn down(&self) -> Option<Self::Neighbor> {
        self.go(Direction::Down)
    }
    #[must_use]
    fn left(&self) -> Option<Self::Neighbor> {
        self.go(Direction::Left)
    }
    #[must_use]
    fn right(&self) -> Option<Self::Neighbor> {
        self.go(Direction::Right)
    }
}

impl<T: Grid> GridShortcuts for T {}
