use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::coords::Coords;
use crate::coords::PlanarProjection;
use crate::grid::Direction;
use crate::grid::Grid;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct SupergraphElement<Supergraph>
where
    Supergraph: IsPlanarEdgeSupergraph,
{
    pub original: Supergraph,
    pub tile: Supergraph::Original,
}

impl<Supergraph> Hash for SupergraphElement<Supergraph>
where
    Supergraph: IsPlanarEdgeSupergraph,
    Supergraph::Original: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tile.hash(state);
    }
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

#[derive(Debug)]
#[must_use]
pub struct PlanarEdgeSupergraph<T> {
    edges: HashMap<(T, Direction), T>,
}

impl<Tile> PlanarEdgeSupergraph<Tile> {
    fn new_empty() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum AddEdgeError {
    ProjectionNotAdjacent,
    OriginalFromAlreadyConnected,
    OriginalToAlreadyConnected,
    OverwritingFromConnection,
    OverwritingToConnection,
}

impl<T> PlanarEdgeSupergraph<T>
where
    T: Grid<Neighbor = T> + PlanarProjection,
    T: Eq + Hash,
{
    fn add_edge(&mut self, from: &T, to: &T) -> Result<(), AddEdgeError> {
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
        self.edges.insert(forwards, to.clone());
        self.edges.insert(backwards, from.clone());

        Ok(())
    }
}

impl<T> IsPlanarEdgeSupergraph for &PlanarEdgeSupergraph<T>
where
    T: Grid<Neighbor = T> + PlanarProjection,
    T: Eq + Hash,
{
    type Original = T;

    fn get_added_edge(&self, from: &Self::Original, dir: Direction) -> Option<Self::Original> {
        self.edges.get(&(from.clone(), dir)).cloned()
    }
}

impl<T> IsPlanarEdgeSupergraph for Rc<PlanarEdgeSupergraph<T>>
where
    T: Grid<Neighbor = T> + PlanarProjection,
    T: Eq + Hash,
{
    type Original = T;

    fn get_added_edge(&self, from: &Self::Original, dir: Direction) -> Option<Self::Original> {
        self.edges.get(&(from.clone(), dir)).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::PlanarEdgeSupergraph;
    use crate::edge_supergraph::IsPlanarEdgeSupergraph;
    use crate::free_group::FreeGroup;
    use crate::grid::GridShortcuts;
    use crate::induced_subgraph::IndoorsMap;
    use crate::induced_subgraph::IsInducedSubgraph;

    #[test]
    fn go_follow_edge() {
        let free_identity = FreeGroup::new_empty();
        let free_other = FreeGroup::new_empty().up().right().down().unwrap();

        let mut valid_tiles = IndoorsMap::new_empty();
        valid_tiles.0.insert(free_identity);
        valid_tiles.0.insert(free_other);

        {
            let subset_identity = (&valid_tiles).to_grid(&free_identity).unwrap();
            assert_eq!(subset_identity.right(), None);

            let subset_other = (&valid_tiles).to_grid(&free_other).unwrap();
            assert_eq!(subset_other.left(), None);
        }

        let mut supergraph = PlanarEdgeSupergraph::new_empty();
        supergraph
            .add_edge(
                &(&valid_tiles).to_grid(&free_identity).unwrap(),
                &(&valid_tiles).to_grid(&free_other).unwrap(),
            )
            .unwrap();

        let dingus = (&supergraph).to_grid((&valid_tiles).to_grid(&free_identity).unwrap());
        assert_eq!(dingus.right().unwrap().tile.tile, free_other);

        let dingus = (&supergraph).to_grid((&valid_tiles).to_grid(&free_other).unwrap());
        assert_eq!(dingus.left().unwrap().tile.tile, free_identity);
    }
}
