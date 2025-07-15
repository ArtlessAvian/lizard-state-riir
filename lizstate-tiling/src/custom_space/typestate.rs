use crate::custom_space::CustomSpace;
use crate::direction::Direction;
use crate::tiling_graph::SpaceError;
use crate::tiling_graph::TileError;
use crate::walk::WalkIsFull;
use crate::walk::reduced::ReducedWalk;
use crate::walk::traits::IsAWalkPartial;

/// A known representative of a custom space.
/// The space becomes readonly.
pub struct Representative<'a>(&'a CustomSpace, ReducedWalk);
impl<'a> Representative<'a> {
    fn try_new_direct(space: &'a CustomSpace, path: &ReducedWalk) -> Option<Self> {
        if *path == ReducedWalk::new_empty() || space.contained_reps.contains(path) {
            Some(Self(space, *path))
        } else {
            None
        }
    }

    /// The entrypoint. Also see `CustomSpace::try_representative`.
    /// # Errors
    /// Space does not contain the (tile at the end of the path) / (an equivalent path to the given path).
    /// # Panics
    /// Space is in invalid state, value is not a representative.
    pub fn try_new_recursive(
        space: &'a CustomSpace,
        path: &ReducedWalk,
    ) -> Result<Representative<'a>, SpaceError> {
        if let Some(rep) = Representative::try_new_direct(space, path) {
            Ok(rep)
        } else {
            let (prefix, popped) = path
                .pop_copy()
                .expect("tile does not eq the origin, which means its not empty");

            let prefix_rep = Self::try_new_recursive(space, &prefix)?;

            let neighbor = prefix_rep
                .step_into_neighbor(popped)
                .map_err(|TileError::Unrepresentable| SpaceError::NotInSpace)?;
            // Even if `path` is representable, the path cannot be in the space.

            let rep = neighbor.try_into_rep()?;

            Ok(rep)
        }
    }

    /// # Errors
    /// Neighbor of representative path is not representable.
    pub fn step_into_neighbor(self, dir: Direction) -> Result<RepNeighbor<'a>, TileError> {
        match self.1.push_copy(dir) {
            Ok(stepped) => Ok(RepNeighbor(self.0, stepped)),
            Err(WalkIsFull) => Err(TileError::Unrepresentable),
        }
    }

    pub fn unwrap(self) -> ReducedWalk {
        self.1
    }
}

/// A neighbor of a known representative.
/// Possibly a representative itself, or can be converted into a representative, or not in the space.
pub struct RepNeighbor<'a>(&'a CustomSpace, ReducedWalk);
impl<'a> RepNeighbor<'a> {
    /// # Errors
    /// This neighbor is not a representative.
    /// # Panics
    /// Space is invalid, all values of `equivalent_rep` are representatives.
    pub fn try_into_rep(self) -> Result<Representative<'a>, SpaceError> {
        if let Some(rep) = Representative::try_new_direct(self.0, &self.1) {
            Ok(rep)
        } else if let Some(value) = self.0.equivalent_rep.get(&self.1) {
            let rep = Representative::try_new_direct(self.0, value)
                .expect("all values are representatives");
            Ok(rep)
        } else {
            Err(SpaceError::NotInSpace)
        }
    }

    // No unwrap function. It isn't very useful to have a path that might not be in the space.
}

pub struct MutRepresentative<'a>(&'a mut CustomSpace, ReducedWalk);
pub struct MutRepresentativeNeighbor<'a>(&'a mut CustomSpace, ReducedWalk);
