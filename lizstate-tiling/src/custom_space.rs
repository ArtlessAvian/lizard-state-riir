#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "ReducedWalk can grow maybe (and still be copy)"
)]

use core::ops::Deref;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::direction::Direction;
use crate::tiling_graph::IsASpace;
use crate::tiling_graph::IsATile;
use crate::tiling_graph::IsTilingGraph;
use crate::tiling_graph::StepError;
use crate::walk::WalkIsFull;
use crate::walk::reduced::ReducedWalk;
use crate::walk::traits::IsAWalkPartial;

// pub mod shared;
// pub mod tiling_graph;

pub enum CustomSpaceError {
    NotInSpace,
    Unrepresentable,
}

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
    fn step(self, dir: Direction) -> Result<RepNeighbor, CustomSpaceError> {
        match self.0.push_copy(dir) {
            Ok(stepped) => Ok(RepNeighbor(stepped)),
            Err(WalkIsFull) => Err(CustomSpaceError::Unrepresentable),
        }
    }
}

/// Typestate to help reasoning.
struct RepNeighbor(ReducedWalk);
impl RepNeighbor {
    fn try_rep(self, space: &CustomSpace) -> Result<Representative, CustomSpaceError> {
        if space.contained_reps.contains(&self.0) {
            Ok(Representative(self.0))
        } else if let Some(rep) = space.equivalent_rep.get(&self.0).copied() {
            Ok(Representative(rep))
        } else {
            Err(CustomSpaceError::NotInSpace)
        }
    }
}

impl CustomSpace {
    const EMPTY_PATH: ReducedWalk = ReducedWalk::new_empty();
    const THE_ORIGIN_REP: Representative = Representative(Self::EMPTY_PATH);
    const THE_ORIGIN_TILE: CustomSpaceTile = CustomSpaceTile(Self::EMPTY_PATH);

    /// Checks if the tile at the destination has a representative.
    fn try_path_into_rep(&self, path: &ReducedWalk) -> Result<Representative, CustomSpaceError> {
        if *path == Self::EMPTY_PATH || self.contained_reps.contains(path) {
            Ok(Representative(*path))
        } else {
            let (prefix, popped) = path
                .pop_copy()
                .expect("tile does not eq the origin, which means its not empty");

            let prefix_rep = self.try_path_into_rep(&prefix)?;
            let neighbor = prefix_rep.step(popped)?;
            let rep = neighbor.try_rep(self)?;
            Ok(rep)
        }
    }
}

impl IsASpace for CustomSpace {}

impl IsTilingGraph for CustomSpace {
    type Tile = CustomSpaceTile;

    fn get_origin(&self) -> Self::Tile {
        Self::THE_ORIGIN_TILE
    }

    fn step(
        &self,
        _tile: &Self::Tile,
        _dir: crate::direction::Direction,
    ) -> Result<Self::Tile, StepError> {
        // let tentative = tile
        //     .0
        //     .push_copy(dir)
        //     .map_err(|WalkIsFull| StepError::Unrepresentable)?;

        todo!()
    }
}

// Newtype around `Rc<CustomSpace>`.
// Derefs to `CustomSpace`, so you can still access the `IsTilingGraph` trait.
#[derive(Clone)]
pub struct SharedCustomSpace(Rc<CustomSpace>);

impl Deref for SharedCustomSpace {
    type Target = CustomSpace;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
