use crate::custom_space::CustomSpace;
use crate::custom_space::CustomSpaceTile;
use crate::tiling_graph::CanFindArbitraryPath;
use crate::tiling_graph::IsASpace;
use crate::tiling_graph::IsTilingGraph;
use crate::tiling_graph::IsWalkable;
use crate::tiling_graph::SpaceError;
use crate::tiling_graph::StepError;
use crate::walk::reduced::Reduced;
use crate::walk::reduced::ReducedWalk;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkRaw;

impl IsASpace for CustomSpace {}

impl IsTilingGraph for CustomSpace {
    type Tile = CustomSpaceTile;

    fn get_origin(&self) -> Self::Tile {
        CustomSpaceTile(ReducedWalk::new_empty())
    }

    fn step(
        &self,
        tile: &Self::Tile,
        dir: crate::direction::Direction,
    ) -> Result<Self::Tile, StepError> {
        let rep = self
            .tile_to_rep(tile)
            .map_err(|SpaceError::NotInSpace| StepError::ArgumentNotInSpace)?;

        let step_rep = rep
            .try_step(dir)
            .map_err(|SpaceError::NotInSpace| StepError::DestinationNotInSpace)?;

        Ok(CustomSpaceTile(step_rep.unwrap_path()))
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
