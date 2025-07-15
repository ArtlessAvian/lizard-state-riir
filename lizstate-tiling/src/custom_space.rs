#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "ReducedWalk can grow maybe (and still be copy)"
)]

use std::collections::HashMap;
use std::collections::HashSet;

use crate::custom_space::typestate::Representative;
use crate::tiling_graph::IsATile;
use crate::tiling_graph::SpaceError;
use crate::walk::reduced::ReducedWalk;

pub mod builder;
pub mod shared;
pub mod tiling_graph;
/// Typestates, holding references to the `CustomSpace` to ensure validity.
pub mod typestate;

/// A nonconstructive tile, at the end of a path.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CustomSpaceTile(ReducedWalk);
impl IsATile for CustomSpaceTile {}

/// Data supporting a subset of the free group elements, and extra connections between elements.
///
/// We define a "representative" as a path to an element, usually a shortest path, but not necessarily.
#[derive(Default)]
#[must_use]
pub struct CustomSpace {
    /// The representative paths for all valid tile.
    /// There are multiple paths to the same tile, so this set chooses one for each.
    /// (Multiple? More like, infinitely many)
    ///
    /// Invariants:
    /// Every valid tile has exactly one representative.
    /// The origin's representative is the empty path.
    contained_reps: HashSet<ReducedWalk>,

    /// A map from (non-representative path, one longer than a representative) to (a representative).
    /// The key's destination tile is the same as the value's destination tile.
    ///
    /// Invariants:
    /// No key is a representative.
    /// All values are representatives.
    ///
    /// Shortest Path Invariants:
    /// Keys and their values differ in length by at most one.
    equivalent_rep: HashMap<ReducedWalk, ReducedWalk>,
}

impl CustomSpace {
    /// Wraps a tile with a `Representative` typestate for easy manipulation.
    /// If you are unsure if a path leads to a tile, prefer using `IsWalkable::walk_from_origin.`
    /// # Errors
    /// Space does not contain a tile at the end of the path.
    /// (This should usually not happen.)
    pub fn tile_to_rep<'a>(
        &'a self,
        tile: &CustomSpaceTile,
    ) -> Result<Representative<'a>, SpaceError> {
        Representative::try_new_recursive(self, &tile.0)
    }
}
