use crate::isometry::GridIsometry;
use crate::isometry::Orientation;
use crate::point::Point;
use crate::tiles::Grid;
use crate::tiles::PlanarProjection;

// impl<T: Grid<NeighborType = T>> Grid for BoundedPoint<T> {
//     type NeighborType = Self;

//     fn go(&self, dir: Direction) -> Option<Self::NeighborType> {
//         Some(BoundedPoint(self.0.go(dir)?))
//     }
// }

#[derive(Clone, Copy)]
#[must_use]
struct Line<T: Grid> {
    from: T,
    to: T,
}

impl<T: Grid<NeighborType = T> + Clone + PlanarProjection> IntoIterator for Line<T> {
    type Item = Point<T>;
    type IntoIter = LineIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        let from_coord = self.from.project_coords();
        let to_coord = self.from.project_coords();

        let dx = to_coord.0 - from_coord.0;
        let dy = to_coord.1 - from_coord.1;

        let orientation = Orientation::new_from_symmetry((dx, dy));

        let from = GridIsometry::new_from_orientation(self.from, orientation);
        let to = GridIsometry::new_from_orientation(self.to, orientation);

        let rotated_line = Line {
            from: from.clone(),
            to,
        };

        LineIterator {
            rotated_line,
            uninit: true,
            just_returned: None,
        }
    }
}

#[must_use]
struct LineIterator<T: Grid<NeighborType = T> + Clone> {
    rotated_line: Line<GridIsometry<T>>,
    uninit: bool,
    just_returned: Option<GridIsometry<T>>,
}

impl<T: Grid<NeighborType = T> + PlanarProjection + Clone> Iterator for LineIterator<T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.uninit {
            self.uninit = false;
            self.just_returned = Some(self.rotated_line.from.clone());

            // we can unwrap first because we know we're putting dx dy at 0 0
            return self
                .just_returned
                .clone()
                .map(GridIsometry::unwrap)
                .map(|tile| Point {
                    tile,
                    dx: 0f32,
                    dy: 0f32,
                });
        }

        let previous = self.just_returned.as_ref()?;
        let from = self.rotated_line.from.project_coords();
        let along = previous.project_coords();
        let to = self.rotated_line.to.project_coords();

        if along == to {
            // We're done.
            self.just_returned = None;
            return None;
        }

        if (2 * along.1 + 1) * (to.0 - from.0) < (2 * along.0 + 1 - 2 * from.0) * (to.1 - from.1) {
            self.just_returned = previous.up();
            return self
                .just_returned
                .clone()
                .map(|tile| Point {
                    tile,
                    dx: 0f32, // TODO
                    dy: -0.5f32,
                })
                .as_ref()
                .map(Point::transpose)
                .map(GridIsometry::unwrap);
        }

        self.just_returned = previous.right();
        self.just_returned
            .clone()
            .map(|tile| Point {
                tile,
                dx: -0.5f32,
                dy: 0f32, // TODO
            })
            .as_ref()
            .map(Point::transpose)
            .map(GridIsometry::unwrap)
    }
}
