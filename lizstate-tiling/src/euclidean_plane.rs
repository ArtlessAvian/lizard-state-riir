use crate::direction::Direction;
use crate::group::GroupOp;
use crate::tiling::HasSquareTiling;
use crate::tiling::IsATile;

/// Z cross Z, though bounded by i32.
///
/// Alternatively, you can think of the `FreeGroup`, but the elements commute!
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

// This is the only abelian group that can be generated with two (non-overlapping) elements.
pub struct PairwiseAddition;

impl GroupOp for PairwiseAddition {
    type Element = CartesianCoords;

    const IDENTITY: Self::Element = CartesianCoords { x: 0, y: 0 };
    const UP: Self::Element = CartesianCoords { x: 0, y: 1 };
    const RIGHT: Self::Element = CartesianCoords { x: 1, y: 0 };

    fn inverse(&self, a: &Self::Element) -> Self::Element {
        CartesianCoords { x: -a.x, y: -a.y }
    }

    fn op(&self, a: &Self::Element, b: &Self::Element) -> Option<Self::Element> {
        Some(CartesianCoords {
            x: a.x.checked_add(b.x)?,
            y: a.y.checked_add(b.y)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::direction::Direction;
    use crate::euclidean_plane::CartesianCoords;
    use crate::euclidean_plane::EuclideanPlane;
    use crate::euclidean_plane::PairwiseAddition;
    use crate::group::GroupOp;
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

    #[test]
    fn vector_addition() {
        let vec_a = CartesianCoords::new(10, 20);
        let vec_b = CartesianCoords::new(30, 40);

        let sum = PairwiseAddition.op(&vec_a, &vec_b);
        assert_eq!(sum, Some(CartesianCoords::new(40, 60)));

        let diff = PairwiseAddition.op(&vec_a, &PairwiseAddition.inverse(&vec_b));
        assert_eq!(diff, Some(CartesianCoords::new(-20, -20)));
    }
}
