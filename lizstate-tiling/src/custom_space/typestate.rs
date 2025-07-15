use crate::custom_space::CustomSpace;
use crate::direction::Direction;
use crate::tiling_graph::SpaceError;
use crate::tiling_graph::TileError;
use crate::walk::WalkIsFull;
use crate::walk::reduced::ReducedWalk;
use crate::walk::traits::IsAWalkPartial;

/// Typestate to help reasoning.
pub struct Representative(pub ReducedWalk);
impl Representative {
    /// # Errors
    /// Neighbor of representative is not representable.
    pub fn step(self, dir: Direction) -> Result<RepNeighbor, TileError> {
        match self.0.push_copy(dir) {
            Ok(stepped) => Ok(RepNeighbor(stepped)),
            Err(WalkIsFull) => Err(TileError::Unrepresentable),
        }
    }
}

/// Typestate to help reasoning.
pub struct RepNeighbor(pub ReducedWalk);
impl RepNeighbor {
    /// # Errors
    /// Neighbor of representative is not a representative.
    pub fn try_rep(self, space: &CustomSpace) -> Result<Representative, SpaceError> {
        if space.contained_reps.contains(&self.0) {
            Ok(Representative(self.0))
        } else if let Some(rep) = space.equivalent_rep.get(&self.0).copied() {
            Ok(Representative(rep))
        } else {
            Err(SpaceError::NotInSpace)
        }
    }
}
