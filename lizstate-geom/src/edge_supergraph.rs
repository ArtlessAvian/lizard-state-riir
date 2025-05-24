use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::coords::Coords;
use crate::coords::PlanarProjection;
use crate::grid::Direction;
use crate::grid::Grid;

pub enum AddEdgeError {
    ProjectionNotAdjacent,
    OriginalFromAlreadyConnected,
    OriginalToAlreadyConnected,
    OverwritingFromConnection,
    OverwritingToConnection,
}

pub trait IsPlanarEdgeSupergraph: Clone {
    type Original: Grid<Neighbor = Self::Original> + PlanarProjection;

    // There does not exist an alternate implementation.
    fn to_grid(&self, tile: Self::Original) -> SupergraphElement<Self> {
        SupergraphElement {
            original: self.clone(),
            tile,
        }
    }

    /// This will only be called when `from` does not have a `dir` neighbor.
    /// This only needs to return Some for edges not already in the graph.
    ///
    /// Implementers, be sure that `get_edge(get_edge(from, dir)?, dir.inverse()) == from`
    fn get_added_edge(&self, from: &Self::Original, dir: Direction) -> Option<Self::Original>;

    fn add_edge(&mut self, from: &Self::Original, to: &Self::Original) -> Result<(), AddEdgeError>;
}

pub struct SupergraphElement<Supergraph>
where
    Supergraph: IsPlanarEdgeSupergraph,
{
    pub original: Supergraph,
    pub tile: Supergraph::Original,
}

impl<T> Grid for SupergraphElement<T>
where
    T: IsPlanarEdgeSupergraph,
{
    type Neighbor = Self;

    fn go(&self, dir: Direction) -> Option<Self::Neighbor> {
        // We call `go` on the original first.
        // Since the original implements `Grid`, we are sure that `SupergraphElement` follows the rules of the `Grid` trait.

        self.tile
            .go(dir)
            .or_else(|| self.original.get_added_edge(&self.tile, dir))
            .map(|neighbor| SupergraphElement {
                original: self.original.clone(),
                tile: neighbor,
            })
    }
}

impl<T> PlanarProjection for SupergraphElement<T>
where
    T: IsPlanarEdgeSupergraph,
{
    fn project_coords(&self) -> Coords {
        self.tile.project_coords()
    }
}

///////////////////////////////////////

#[derive(Debug, Clone)]
#[must_use]
pub struct PlanarEdgeSupergraph<T> {
    edges: Rc<HashMap<(T, Direction), T>>,
}

impl<Tile> PlanarEdgeSupergraph<Tile> {
    fn new_empty() -> Self {
        Self {
            edges: Rc::new(HashMap::new()),
        }
    }
}

impl<T> IsPlanarEdgeSupergraph for PlanarEdgeSupergraph<T>
where
    T: Grid<Neighbor = T> + PlanarProjection,
    T: Eq + Hash,
    T: Clone,
{
    type Original = T;

    fn get_added_edge(&self, from: &Self::Original, dir: Direction) -> Option<Self::Original> {
        // TODO: Avoid cloning input >:/
        self.edges.get(&(from.clone(), dir)).cloned()
    }

    fn add_edge(&mut self, from: &Self::Original, to: &Self::Original) -> Result<(), AddEdgeError> {
        let from_coords = from.project_coords();
        let to_coords = to.project_coords();

        // This check makes it planar.
        let dir = match ((to_coords.x - from_coords.x), (to_coords.y - from_coords.y)) {
            (0, 1) => Direction::Up,
            (0, -1) => Direction::Down,
            (-1, 0) => Direction::Left,
            (1, 0) => Direction::Right,
            _ => return Err(AddEdgeError::ProjectionNotAdjacent),
        };

        if from.go(dir).is_some() {
            return Err(AddEdgeError::OriginalFromAlreadyConnected);
        }
        if to.go(dir.inverse()).is_some() {
            return Err(AddEdgeError::OriginalToAlreadyConnected);
        }

        let forwards = (from.clone(), dir);
        let backwards = (to.clone(), dir.inverse());

        if self.edges.contains_key(&forwards) {
            return Err(AddEdgeError::OverwritingFromConnection);
        }
        if self.edges.contains_key(&backwards) {
            return Err(AddEdgeError::OverwritingToConnection);
        }

        // TODO: Remove unwrap! Use inner struct!
        Rc::get_mut(&mut self.edges)
            .unwrap()
            .insert(forwards, to.clone());
        Rc::get_mut(&mut self.edges)
            .unwrap()
            .insert(backwards, from.clone());

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::PlanarEdgeSupergraph;
//     use crate::free_group::FreeGroup;
//     use crate::layered_map::SewingError;
//     use crate::tiles::Grid;
//     use crate::tiles::Tile;

//     #[test]
//     fn locate() {
//         let mut map = PlanarEdgeSupergraph::new_empty();

//         let tile = FreeGroup::new_empty();
//         let tile_right = tile.right().unwrap();

//         map.tiles.insert(tile, ());

//         assert!(map.locate(&tile).is_some());
//         assert!(map.locate(&tile_right).is_none());

//         map.tiles.insert(tile_right, ());

//         assert!(map.locate(&tile_right).is_some());
//     }

//     #[test]
//     fn add_connections() {
//         let mut map = PlanarEdgeSupergraph::new_empty();

//         let tile = FreeGroup::new_empty();
//         let tile_right_down_left = tile.right().down().left().unwrap();
//         map.tiles.insert(tile, ());
//         map.tiles.insert(tile.right().unwrap(), ());
//         map.tiles.insert(tile.right().down().unwrap(), ());
//         map.tiles.insert(tile_right_down_left, ());

//         let in_map = map.locate(&tile).unwrap();
//         assert!(in_map.down().is_none());
//         drop(in_map);

//         assert_eq!(map.insert_edge(&tile, &tile_right_down_left), Ok(()));

//         let in_map = map.locate(&tile).unwrap();
//         assert_eq!(*in_map.down().unwrap().tile, tile_right_down_left);
//         assert_eq!(*in_map.down().up().unwrap().tile, *in_map.tile);
//     }

//     #[test]
//     fn reject_invalid_connections() {
//         let mut map = PlanarEdgeSupergraph::new_empty();

//         let tile = Tile::new(0, 0);
//         let tile_right_right = tile.right().right().unwrap();

//         assert_eq!(
//             map.insert_edge(&tile, &tile_right_right),
//             Err(SewingError::MissingFrom)
//         );
//         map.tiles.insert(tile, ());

//         assert_eq!(
//             map.insert_edge(&tile, &tile_right_right),
//             Err(SewingError::MissingTo)
//         );
//         map.tiles.insert(tile_right_right, ());

//         assert_eq!(
//             map.insert_edge(&tile, &tile_right_right),
//             Err(SewingError::FromToNotAdjacent)
//         );
//     }
// }
