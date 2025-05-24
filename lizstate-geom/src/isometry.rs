use crate::tiles::Direction;
use crate::tiles::Grid;
use crate::tiles::PlanarProjection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum Orientation {
    UpRight,   // The Identity
    UpLeft,    // Flip X = 0
    DownRight, // Flip Y = 0
    DownLeft,  // Rotate 180 degrees
    LeftUp,    // Rotate 90 degrees counterclockwise
    LeftDown,  // Flip Y = -X
    RightUp,   // Flip Y = X
    RightDown, // Rotate 90 degrees clockwise
}

impl Orientation {
    fn new_from_dirs(up: Direction, right: Direction) -> Option<Self> {
        match (up, right) {
            (Direction::Up, Direction::Left) => Some(Self::UpLeft),
            (Direction::Up, Direction::Right) => Some(Self::UpRight),
            (Direction::Down, Direction::Left) => Some(Self::DownLeft),
            (Direction::Down, Direction::Right) => Some(Self::DownRight),
            (Direction::Left, Direction::Up) => Some(Self::LeftUp),
            (Direction::Left, Direction::Down) => Some(Self::LeftDown),
            (Direction::Right, Direction::Up) => Some(Self::RightUp),
            (Direction::Right, Direction::Down) => Some(Self::RightDown),
            _ => None,
        }
    }

    fn to_pair(self) -> (Direction, Direction) {
        match self {
            Orientation::UpRight => (Direction::Up, Direction::Right),
            Orientation::UpLeft => (Direction::Up, Direction::Left),
            Orientation::DownRight => (Direction::Down, Direction::Right),
            Orientation::DownLeft => (Direction::Down, Direction::Left),
            Orientation::LeftUp => (Direction::Left, Direction::Up),
            Orientation::LeftDown => (Direction::Left, Direction::Down),
            Orientation::RightUp => (Direction::Right, Direction::Up),
            Orientation::RightDown => (Direction::Right, Direction::Down),
        }
    }

    /// "right" is the direction with most contribution.
    /// "up" is the direction with second most contribution
    /// This puts the vec inside the octant x >= 0, y >= 0, y <= x.
    #[expect(clippy::missing_panics_doc, reason = "expect")]
    pub fn new_from_symmetry((mut x, mut y): (i32, i32)) -> Self {
        let mut up = Direction::Up;
        let mut right = Direction::Right;

        if x < 0 {
            right = right.inverse();
            x *= -1;
        }
        if y < 0 {
            up = up.inverse();
            y *= -1;
        }
        if x < y {
            (up, right) = (right, up);
            (x, y) = (y, x);
        }

        let (_, _) = (x, y);
        Self::new_from_dirs(up, right).expect("no operation here makes up and right colinear")
    }

    fn to_up(self) -> Direction {
        self.to_pair().0
    }

    fn to_right(self) -> Direction {
        self.to_pair().1
    }

    pub fn inverse(self) -> Self {
        match self {
            Orientation::RightDown => Self::LeftUp,
            Orientation::LeftUp => Self::RightDown,
            // All other orientations are flips, 180 degree rotations, and the identity
            // These are their own identity
            _ => self,
        }
    }

    fn local_to_inner(self, (x, y): (i32, i32)) -> (i32, i32) {
        match self {
            Self::UpLeft => (-x, y),
            Self::UpRight => (x, y),
            Self::DownLeft => (-x, -y),
            Self::DownRight => (x, -y),
            Self::LeftUp => (-y, x),
            Self::LeftDown => (-y, -x),
            Self::RightUp => (y, x),
            Self::RightDown => (y, -x),
        }
    }

    fn inner_to_local(self, (x, y): (i32, i32)) -> (i32, i32) {
        self.inverse().local_to_inner((x, y))
    }

    fn local_direction_to_inner(self, dir: Direction) -> Direction {
        match dir {
            Direction::Up => self.to_up(),
            Direction::Down => self.to_up().inverse(),
            Direction::Left => self.to_right().inverse(),
            Direction::Right => self.to_right(),
        }
    }

    fn inner_direction_to_local(self, dir: Direction) -> Direction {
        self.inverse().local_direction_to_inner(dir)
    }
}

/// Moving up on this grid moves `Isometry::Up` instead.
///
/// Moving right on this grid moves perpendicular.
/// This direction is clockwise (flip is false) or counterclockwise (flip is true).
#[derive(Clone, Copy)]
#[must_use]
pub struct GridIsometry<T: Grid> {
    pub tile: T,
    pub orientation: Orientation,
}

impl<T: Grid> GridIsometry<T> {
    pub fn new_from_symmetry(tile: T, (x, y): (i32, i32)) -> GridIsometry<T> {
        GridIsometry {
            tile,
            orientation: Orientation::new_from_symmetry((x, y)),
        }
    }

    pub fn new_from_orientation(tile: T, orientation: Orientation) -> GridIsometry<T> {
        GridIsometry { tile, orientation }
    }

    fn new_from_dirs(tile: T, up: Direction, right: Direction) -> Option<GridIsometry<T>> {
        Some(GridIsometry {
            tile,
            orientation: Orientation::new_from_dirs(up, right)?,
        })
    }

    pub fn unwrap(self) -> T {
        self.tile
    }
}

impl<T: Grid<NeighborType = T>> Grid for GridIsometry<T> {
    type NeighborType = Self;

    fn go(&self, dir: Direction) -> Option<Self::NeighborType> {
        self.tile
            .go(self.orientation.local_direction_to_inner(dir))
            .map(|tile| GridIsometry {
                tile,
                orientation: self.orientation,
            })
    }
}

/// We must follow the description as given by the trait.
/// Calling `Grid::up` must increase the y value by 1.
/// Calling `Grid::right` must increase the x value by 1.
///
/// We know T corectly implements the trait.
/// When we call `Grid::up`, it's as if we called some other direction.
/// We need to turn that direction back into up.
impl<T: Grid<NeighborType = T> + PlanarProjection> PlanarProjection for GridIsometry<T> {
    fn project_coords(&self) -> (i32, i32) {
        self.orientation.inner_to_local(self.tile.project_coords())
    }
}

#[cfg(test)]
mod tests {
    use super::Orientation;
    use crate::isometry::GridIsometry;
    use crate::tiles::Direction;
    use crate::tiles::Grid;
    use crate::tiles::Tile;

    #[test]
    fn symmetry() {
        for (vec, expected) in [
            ((2, 1), Orientation::UpRight),
            ((-2, 1), Orientation::UpLeft),
            ((2, -1), Orientation::DownRight),
            ((-2, -1), Orientation::DownLeft),
            ((-1, 2), Orientation::LeftUp),
            ((-1, -2), Orientation::LeftDown),
            ((1, 2), Orientation::RightUp),
            ((1, -2), Orientation::RightDown),
        ] {
            let orientation = Orientation::new_from_symmetry(vec);
            assert_eq!(orientation, expected);

            println!("testing {expected:?}");
            assert_eq!(orientation.local_to_inner((2, 1)), vec);
            assert_eq!(orientation.inner_to_local(vec), (2, 1));

            let iso = GridIsometry::new_from_symmetry(Tile::new(0, 0), vec);
            assert_eq!(iso.orientation, expected);
            assert_eq!(
                iso.right().right().up().unwrap().tile,
                Tile::new(vec.0, vec.1)
            );
        }
    }

    #[test]
    fn directions() {
        for orientation in [
            Orientation::UpRight,
            Orientation::UpLeft,
            Orientation::DownRight,
            Orientation::DownLeft,
            Orientation::LeftUp,
            Orientation::LeftDown,
            Orientation::RightUp,
            Orientation::RightDown,
        ] {
            match orientation {
                Orientation::UpLeft | Orientation::UpRight => {
                    assert_eq!(orientation.to_up(), Direction::Up);
                }
                Orientation::DownRight | Orientation::DownLeft => {
                    assert_eq!(orientation.to_up(), Direction::Down);
                }
                Orientation::LeftUp | Orientation::LeftDown => {
                    assert_eq!(orientation.to_up(), Direction::Left);
                }
                Orientation::RightUp | Orientation::RightDown => {
                    assert_eq!(orientation.to_up(), Direction::Right);
                }
            }

            match orientation {
                Orientation::UpLeft | Orientation::DownLeft => {
                    assert_eq!(orientation.to_right(), Direction::Left);
                }
                Orientation::DownRight | Orientation::UpRight => {
                    assert_eq!(orientation.to_right(), Direction::Right);
                }
                Orientation::LeftUp | Orientation::RightUp => {
                    assert_eq!(orientation.to_right(), Direction::Up);
                }
                Orientation::LeftDown | Orientation::RightDown => {
                    assert_eq!(orientation.to_right(), Direction::Down);
                }
            }

            assert_eq!(
                Orientation::new_from_dirs(orientation.to_up(), orientation.to_right()),
                Some(orientation)
            );
        }
    }
}
