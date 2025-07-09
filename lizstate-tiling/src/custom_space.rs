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
