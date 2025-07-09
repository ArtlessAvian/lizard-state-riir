#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "ReducedWalk can grow maybe (and still be copy)"
)]

use std::collections::HashMap;
use std::collections::HashSet;

use crate::direction::Direction;
use crate::tiling_graph::IsATile;
use crate::tiling_graph::SpaceError;
use crate::tiling_graph::TileError;
use crate::walk::WalkIsFull;
use crate::walk::reduced::ReducedWalk;
use crate::walk::traits::IsAWalkPartial;

pub mod builder;
pub mod shared;
pub mod tiling_graph;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CustomSpaceTile(ReducedWalk);
impl IsATile for CustomSpaceTile {}

/// Data supporting extra connections in the free group.
#[derive(Default)]
#[must_use]
pub struct CustomSpace {
    /// The representative paths for all valid locations.
    /// There are multiple paths to the same location,
    /// so this set chooses one for each.
    /// (Multiple? More like, infinitely many)
    contained_reps: HashSet<ReducedWalk>,
    /// A map from (non-representative path, one longer than a representative) to (a representative).
    /// Paths two longer than a representative are not stored.
    /// Ideally, the value is shorter than the key.
    equivalent_rep: HashMap<ReducedWalk, ReducedWalk>,
}

/// Typestate to help reasoning.
struct Representative(ReducedWalk);
impl Representative {
    fn step(self, dir: Direction) -> Result<RepNeighbor, TileError> {
        match self.0.push_copy(dir) {
            Ok(stepped) => Ok(RepNeighbor(stepped)),
            Err(WalkIsFull) => Err(TileError::Unrepresentable),
        }
    }
}

/// Typestate to help reasoning.
struct RepNeighbor(ReducedWalk);
impl RepNeighbor {
    fn try_rep(self, space: &CustomSpace) -> Result<Representative, SpaceError> {
        if space.contained_reps.contains(&self.0) {
            Ok(Representative(self.0))
        } else if let Some(rep) = space.equivalent_rep.get(&self.0).copied() {
            Ok(Representative(rep))
        } else {
            Err(SpaceError::NotInSpace)
        }
    }
}

impl CustomSpace {
    const EMPTY_PATH: ReducedWalk = ReducedWalk::new_empty();
    const THE_ORIGIN_REP: Representative = Representative(Self::EMPTY_PATH);
    const THE_ORIGIN_TILE: CustomSpaceTile = CustomSpaceTile(Self::EMPTY_PATH);

    /// Checks if the tile at the destination has a representative.
    fn try_path_into_rep(&self, path: &ReducedWalk) -> Result<Representative, SpaceError> {
        if *path == Self::EMPTY_PATH || self.contained_reps.contains(path) {
            Ok(Representative(*path))
        } else {
            let (prefix, popped) = path
                .pop_copy()
                .expect("tile does not eq the origin, which means its not empty");

            let prefix_rep = self.try_path_into_rep(&prefix)?;

            let neighbor = prefix_rep
                .step(popped)
                .map_err(|TileError::Unrepresentable| SpaceError::NotInSpace)?;
            // Even if `path` is representable, the path cannot be in the space.

            let rep = neighbor.try_rep(self)?;

            Ok(rep)
        }
    }
}
