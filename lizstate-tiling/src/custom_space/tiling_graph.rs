use crate::custom_space::CustomSpace;
use crate::custom_space::CustomSpaceTile;
use crate::tiling_graph::CanFindArbitraryPath;
use crate::tiling_graph::IsASpace;
use crate::tiling_graph::IsTilingGraph;
use crate::tiling_graph::IsWalkable;
use crate::tiling_graph::SpaceError;
use crate::tiling_graph::StepError;
use crate::tiling_graph::TileError;
use crate::walk::reduced::Reduced;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkRaw;

impl IsASpace for CustomSpace {}

impl IsTilingGraph for CustomSpace {
    type Tile = CustomSpaceTile;

    fn get_origin(&self) -> Self::Tile {
        Self::THE_ORIGIN_TILE
    }

    fn step(
        &self,
        tile: &Self::Tile,
        dir: crate::direction::Direction,
    ) -> Result<Self::Tile, StepError> {
        let rep = self
            .try_path_into_rep(&tile.0)
            .map_err(|SpaceError::NotInSpace| StepError::ArgumentNotInSpace)?;

        let neighbor = rep
            .step(dir)
            .map_err(|TileError::Unrepresentable| StepError::DestinationUnrepresentable)?;

        let step_rep = neighbor
            .try_rep(self)
            .map_err(|SpaceError::NotInSpace| StepError::DestinationNotInSpace)?;

        Ok(CustomSpaceTile(step_rep.0))
    }
}

impl IsWalkable for CustomSpace {}

impl CanFindArbitraryPath for CustomSpace {
    fn path_from_origin<Walk: IsAWalkRaw>(
        &self,
        to: &Self::Tile,
    ) -> Result<Reduced<Walk>, Walk::PushError> {
        Reduced::<Walk>::try_new_from_iter(to.0)
    }
}
