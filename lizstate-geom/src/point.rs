use crate::isometry::GridIsometry;
use crate::isometry::Orientation;
use crate::tiles::Direction;
use crate::tiles::Grid;
use crate::tiles::PlanarProjection;

/// A tile of side length 1, and an offset.
///
/// The offset is within the tile for (dx, dy) in [-0.5, 0.5) x [-0.5, 0.5)
/// The offset does not *need* to be within the tile.
#[derive(Clone, Copy)]
#[must_use]
pub struct Point<T: Grid> {
    pub tile: T,
    pub dx: f32, // - left right +
    pub dy: f32, // - down up +
}

impl<T: Grid<NeighborType = T>> Grid for Point<T> {
    type NeighborType = Self;

    fn go(&self, dir: Direction) -> Option<Self::NeighborType> {
        Some(Point {
            tile: self.tile.go(dir)?,
            ..*self
        })
    }
}

impl<T: Grid<NeighborType = T> + PlanarProjection> PlanarProjection for Point<T> {
    fn project_coords(&self) -> (i32, i32) {
        self.tile.project_coords()
    }
}

impl<T: Grid<NeighborType = T> + Clone> Point<GridIsometry<T>> {
    pub fn transpose(&self) -> GridIsometry<Point<T>> {
        let (dx, dy) = match self.tile.orientation {
            Orientation::UpRight => (self.dx, self.dy),
            Orientation::UpLeft => (-self.dx, self.dy),
            Orientation::DownRight => (self.dx, -self.dy),
            Orientation::DownLeft => (-self.dx, -self.dy),
            Orientation::LeftUp => (-self.dy, self.dx),
            Orientation::LeftDown => (-self.dy, -self.dx),
            Orientation::RightUp => (self.dy, self.dx),
            Orientation::RightDown => (self.dy, -self.dx),
        };

        GridIsometry {
            tile: Point {
                tile: self.tile.tile.clone(),
                dx,
                dy,
            },
            orientation: self.tile.orientation,
        }
    }
}

impl<T: Grid<NeighborType = T> + Clone> GridIsometry<Point<T>> {
    pub fn transpose(&self) -> Point<GridIsometry<T>> {
        let (dx, dy) = match self.orientation.inverse() {
            Orientation::UpRight => (self.tile.dx, self.tile.dy),
            Orientation::UpLeft => (-self.tile.dx, self.tile.dy),
            Orientation::DownRight => (self.tile.dx, -self.tile.dy),
            Orientation::DownLeft => (-self.tile.dx, -self.tile.dy),
            Orientation::LeftUp => (-self.tile.dy, self.tile.dx),
            Orientation::LeftDown => (-self.tile.dy, -self.tile.dx),
            Orientation::RightUp => (self.tile.dy, self.tile.dx),
            Orientation::RightDown => (self.tile.dy, -self.tile.dx),
        };

        Point {
            tile: GridIsometry {
                tile: self.tile.tile.clone(),
                orientation: self.orientation,
            },
            dx,
            dy,
        }
    }
}
