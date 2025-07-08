use crate::direction::Direction;
use crate::tiling::HasSquareTiling;
use crate::tiling::IsATile;

pub mod impl_isagroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct CartesianCoords {
    x: i32,
    y: i32,
}

impl CartesianCoords {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl IsATile for CartesianCoords {}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct EuclideanPlane;

impl HasSquareTiling<CartesianCoords> for EuclideanPlane {
    fn get_origin(&self) -> CartesianCoords {
        CartesianCoords { x: 0, y: 0 }
    }

    fn step(&self, tile: &CartesianCoords, dir: Direction) -> Option<CartesianCoords>
    where
        CartesianCoords: Clone,
    {
        Some(match dir {
            Direction::Up => CartesianCoords {
                x: tile.x,
                y: tile.y.checked_add(1)?,
            },
            Direction::Down => CartesianCoords {
                x: tile.x,
                y: tile.y.checked_add(-1)?,
            },
            Direction::Right => CartesianCoords {
                x: tile.x.checked_add(1)?,
                y: tile.y,
            },
            Direction::Left => CartesianCoords {
                x: tile.x.checked_add(-1)?,
                y: tile.y,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::direction::Direction;
    use crate::euclidean_plane::EuclideanPlane;
    use crate::tiling::HasSquareTiling;

    #[test]
    fn space_traversal() {
        let origin = EuclideanPlane.get_origin();

        let up_right_down_left = EuclideanPlane.skip_path(
            &origin,
            [
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ],
        );

        assert_eq!(up_right_down_left, Ok(origin));
    }
}
