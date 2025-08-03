use core::result::Result;

use crate::coords::CartesianCoords;
use crate::direction::Direction;
use crate::tiling_graph::CanFindArbitraryPath;
use crate::tiling_graph::IsASpace;
use crate::tiling_graph::IsATile;
use crate::tiling_graph::IsTilingGraph;
use crate::tiling_graph::IsWalkable;
use crate::tiling_graph::StepError;
use crate::walk::reduced::Reduced;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkMut;
use crate::walk::traits::IsAWalkRaw;

impl IsATile for CartesianCoords {}

pub struct TheEuclideanPlane;

impl IsASpace for TheEuclideanPlane {}

impl IsTilingGraph for TheEuclideanPlane {
    // Ok, If you want to be PEDANTIC about it, this isnt *the* Euclidean plane.
    // Tile would need to be `(BigInt, BigInt)`.
    type Tile = CartesianCoords;

    fn get_origin(&self) -> Self::Tile {
        CartesianCoords::new(0, 0)
    }

    fn step(&self, tile: &Self::Tile, dir: Direction) -> Result<CartesianCoords, StepError> {
        tile.step(dir).ok_or(StepError::DestinationUnrepresentable)
    }
}

impl IsWalkable for TheEuclideanPlane {}

impl CanFindArbitraryPath for TheEuclideanPlane {
    fn path_from_origin<Walk: IsAWalkRaw>(
        &self,
        to: &Self::Tile,
    ) -> Result<Reduced<Walk>, Walk::PushError> {
        let out = Reduced::<Walk>::new_empty();
        let out = out.try_extend((0..to.x).map(|_| Direction::Right))?;
        let out = out.try_extend((0..-to.x).map(|_| Direction::Left))?;
        let out = out.try_extend((0..to.y).map(|_| Direction::Up))?;
        let out = out.try_extend((0..-to.y).map(|_| Direction::Down))?;
        Ok(out)
    }

    fn path_between_tiles<Walk: IsAWalkRaw>(
        &self,
        from: &Self::Tile,
        to: &Self::Tile,
    ) -> Result<Reduced<Walk>, Walk::PushError> {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        self.path_from_origin(&CartesianCoords::new(dx, dy))
    }
}

#[cfg(test)]
mod tests {
    use crate::coords::CartesianCoords;
    use crate::direction::Direction;
    use crate::euclidean_plane::TheEuclideanPlane;
    use crate::tiling_graph::CanFindArbitraryPath;
    use crate::tiling_graph::IsTilingGraph;
    use crate::tiling_graph::IsWalkable;
    use crate::walk::reduced::ReducedWalkEnum;
    use crate::walk::traits::IsAWalk;

    #[test]
    fn space_traversal() {
        let origin = TheEuclideanPlane.get_origin();

        let up_right_down_left = TheEuclideanPlane.walk_from_origin([
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ]);

        assert_eq!(up_right_down_left.unwrap(), origin);
    }

    #[test]
    fn find_arbitrary_path() {
        let empty: ReducedWalkEnum = TheEuclideanPlane
            .path_between_tiles(&CartesianCoords::new(3, 3), &CartesianCoords::new(3, 3))
            .unwrap();

        assert!(empty.is_empty());

        let right_up: ReducedWalkEnum = TheEuclideanPlane
            .path_from_origin(&CartesianCoords::new(1, 2))
            .unwrap();

        for (actual, expected) in
            right_up
                .into_iter()
                .zip([Direction::Right, Direction::Up, Direction::Up])
        {
            assert_eq!(actual, expected);
        }

        let left_down: ReducedWalkEnum = TheEuclideanPlane
            .path_from_origin(&CartesianCoords::new(-1, -2))
            .unwrap();

        for (actual, expected) in
            left_down
                .into_iter()
                .zip([Direction::Left, Direction::Down, Direction::Down])
        {
            assert_eq!(actual, expected);
        }
    }
}
