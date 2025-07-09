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
use crate::walk::traits::IsAWalkPartial;
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
        tile.step(dir).ok_or(StepError::Unrepresentable)
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

// use crate::direction::Direction;
// use crate::tiling::HasSquareTiling;
// use crate::tiling::IsATile;

// pub mod impl_isagroup;

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// #[must_use]
// pub struct CartesianCoords {
//     x: i32,
//     y: i32,
// }

// impl CartesianCoords {
//     pub fn new(x: i32, y: i32) -> Self {
//         Self { x, y }
//     }
// }

// impl IsATile for CartesianCoords {}

// #[derive(Default, Clone, Copy, PartialEq, Eq)]
// pub struct EuclideanPlane;

// impl HasSquareTiling<CartesianCoords> for EuclideanPlane {
//     fn get_origin(&self) -> CartesianCoords {
//         CartesianCoords { x: 0, y: 0 }
//     }

//     fn step(&self, tile: &CartesianCoords, dir: Direction) -> Option<CartesianCoords>
//     where
//         CartesianCoords: Clone,
//     {
//         Some(match dir {
//             Direction::Up => CartesianCoords {
//                 x: tile.x,
//                 y: tile.y.checked_add(1)?,
//             },
//             Direction::Down => CartesianCoords {
//                 x: tile.x,
//                 y: tile.y.checked_add(-1)?,
//             },
//             Direction::Right => CartesianCoords {
//                 x: tile.x.checked_add(1)?,
//                 y: tile.y,
//             },
//             Direction::Left => CartesianCoords {
//                 x: tile.x.checked_add(-1)?,
//                 y: tile.y,
//             },
//         })
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::direction::Direction;
//     use crate::euclidean_plane::EuclideanPlane;
//     use crate::tiling::HasSquareTiling;

//     #[test]
//     fn space_traversal() {
//         let origin = EuclideanPlane.get_origin();

//         let up_right_down_left = EuclideanPlane.skip_path(
//             &origin,
//             [
//                 Direction::Up,
//                 Direction::Right,
//                 Direction::Down,
//                 Direction::Left,
//             ],
//         );

//         assert_eq!(up_right_down_left, Ok(origin));
//     }
// }
