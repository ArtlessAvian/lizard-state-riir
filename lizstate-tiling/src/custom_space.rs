use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::tiling_graph::IsASpace;
use crate::tiling_graph::IsATile;
use crate::tiling_graph::IsTilingGraph;
use crate::tiling_graph::StepError;
use crate::walk::reduced::ReducedWalk;

// pub mod shared;
// pub mod tiling_graph;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CustomSpaceRepresentative(ReducedWalk);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CustomSpaceTile(ReducedWalk);
impl IsATile for CustomSpaceTile {}

/// Data supporting extra connections in the free group.
struct CustomSpace {
    /// The representative tiles for all valid locations.
    /// Multiple tiles may represent the same location,
    /// so this set chooses one for each.
    /// (Multiple? More like, infinitely many)
    /// Always contains `CustomSpaceInner::THE_ORIGIN`.
    contained_rep: HashSet<CustomSpaceTile>,
    /// A map from (non-representative tile adjacent to a representative) to (a representative).
    /// Tiles two steps away are not present.
    /// Ideally, the value is shorter than the key.
    equivalent_rep: HashMap<CustomSpaceTile, CustomSpaceTile>,
}

impl CustomSpace {
    const THE_ORIGIN: CustomSpaceTile = CustomSpaceTile(ReducedWalk::new_empty());

    pub fn new() -> Self {
        let mut contained_rep = HashSet::new();
        let equivalent_rep = HashMap::new();

        contained_rep.insert(Self::THE_ORIGIN);

        Self {
            contained_rep,
            equivalent_rep,
        }
    }
}

/// The free group, with extra connections. Assume readonly!
#[derive(Clone)]
pub struct SharedCustomSpace(Rc<CustomSpace>);

impl PartialEq for SharedCustomSpace {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for SharedCustomSpace {}

impl IsASpace for SharedCustomSpace {}

impl IsTilingGraph for SharedCustomSpace {
    type Tile = CustomSpaceTile;

    fn get_origin(&self) -> Self::Tile {
        CustomSpaceTile(ReducedWalk::new_empty())
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

// //! From a space, we want to select a subset of tiles from that space, and add connections between tiles in that space.

// use std::collections::HashMap;
// use std::collections::HashSet;
// use std::rc::Rc;

// use crate::path::BoundedPathLike;
// use crate::path::bits_backed::PathBitString;
// use crate::path::efficient::Efficient;
// use crate::tiling::HasSquareTiling;
// use crate::tiling::IsASpace;
// use crate::tiling::IsATile;

// // #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
// type PathInCustomSpace = Efficient<PathBitString>;
// impl IsATile for PathInCustomSpace {}

// struct CustomSpaceInner {
//     // A set of paths. These form a trie / prefix list.
//     preferred_paths: HashSet<PathInCustomSpace>,
//     // A map of paths (not in the trie) to paths in the trie.
//     // The value's length should always be shorter than the key.
//     forced_connections: HashMap<PathInCustomSpace, PathInCustomSpace>,
// }

// #[derive(Clone)]
// struct CustomSpace(Rc<CustomSpaceInner>);

// impl PartialEq for CustomSpace {
//     fn eq(&self, other: &Self) -> bool {
//         Rc::ptr_eq(&self.0, &other.0)
//     }
// }

// impl Eq for CustomSpace {}

// impl IsASpace for CustomSpace {}

// impl HasSquareTiling<PathInCustomSpace> for CustomSpace {
//     fn get_origin(&self) -> PathInCustomSpace {
//         PathInCustomSpace::default()
//     }

//     fn step(
//         &self,
//         tile: &PathInCustomSpace,
//         dir: crate::direction::Direction,
//     ) -> Option<PathInCustomSpace> {
//         let tentative = tile.push(dir)?;

//         let mut prefix = PathInCustomSpace::new();
//         for yea in tentative {
//             prefix = prefix.push(yea)?;
//             if self.0.preferred_paths.contains(&prefix) {
//                 continue;
//             }
//             if let Some(preferred) = self.0.forced_connections.get(&prefix) {
//                 prefix = *preferred;
//                 continue;
//             }
//             return None;
//         }

//         Some(prefix)
//     }
// }
