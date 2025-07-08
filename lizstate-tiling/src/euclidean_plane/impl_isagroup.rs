use crate::euclidean_plane::CartesianCoords;
use crate::group::IsAGroup;

// This is the only abelian group that can be generated with two (non-overlapping) elements.
pub struct PairwiseAddition;

impl IsAGroup for PairwiseAddition {
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
    use crate::euclidean_plane::impl_isagroup::PairwiseAddition;
    use crate::group::IsAGroup;
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
