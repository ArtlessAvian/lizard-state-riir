use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use crate::coords::Coords;
use crate::coords::PlanarProjection;
use crate::grid::Direction;
use crate::grid::Grid;

/// Implementors are runtime defined induced subgraphs of the tiles. Tiles are still connected as they defined.
///
/// Trait implementors can use `SubsetElement` to traverse the subset, since it implements `Grid`.
pub trait IsInducedSubgraph: Clone {
    type Original: Grid<Neighbor = Self::Original>;

    fn to_grid(&self, tile: &Self::Original) -> Option<InducedSubgraphElement<Self>>;
}

#[derive(Debug, Clone)]
#[must_use]
pub struct InducedSubgraphElement<Subgraph>
where
    Subgraph: IsInducedSubgraph,
{
    pub map: Subgraph,
    pub tile: Subgraph::Original,
}

impl<Subgraph> Hash for InducedSubgraphElement<Subgraph>
where
    Subgraph: IsInducedSubgraph,
    Subgraph::Original: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Ignore self.map.
        self.tile.hash(state);
    }
}

impl<Subgraph> PartialEq for InducedSubgraphElement<Subgraph>
where
    Subgraph: IsInducedSubgraph,
    Subgraph::Original: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.tile == other.tile
    }
}

impl<Subgraph> Eq for InducedSubgraphElement<Subgraph>
where
    Subgraph: IsInducedSubgraph,
    Subgraph::Original: PartialEq,
{
}

impl<Subgraph> Grid for InducedSubgraphElement<Subgraph>
where
    Subgraph: IsInducedSubgraph,
{
    type Neighbor = Self;

    fn go(&self, dir: Direction) -> Option<Self::Neighbor> {
        self.map.clone().to_grid(&self.tile.go(dir)?)
    }
}

impl<Subset> PlanarProjection for InducedSubgraphElement<Subset>
where
    Subset: IsInducedSubgraph,
    Subset::Original: PlanarProjection,
{
    fn project_coords(&self) -> Coords {
        self.tile.project_coords()
    }
}

/// A set of tiles, connected if they exist in the set.
#[derive(Debug, Clone)]
#[must_use]
pub struct IndoorsMap<T>(pub HashSet<T>);

impl<T> IndoorsMap<T> {
    pub fn new_empty() -> Self {
        Self(HashSet::new())
    }
}

impl<T> IsInducedSubgraph for &IndoorsMap<T>
where
    T: Grid<Neighbor = T>,
    T: Hash + Eq,
{
    type Original = T;

    fn to_grid(&self, tile: &Self::Original) -> Option<InducedSubgraphElement<Self>> {
        self.0.get(tile).map(|tile| InducedSubgraphElement {
            map: *self,
            tile: tile.clone(),
        })
    }
}

impl<T> IsInducedSubgraph for Rc<IndoorsMap<T>>
where
    T: Grid<Neighbor = T>,
    T: Hash + Eq,
{
    type Original = T;

    fn to_grid(&self, tile: &Self::Original) -> Option<InducedSubgraphElement<Self>> {
        self.0.get(tile).map(|tile| InducedSubgraphElement {
            map: self.clone(),
            tile: tile.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::IndoorsMap;
    use crate::coords::Coords;
    use crate::grid::GridShortcuts;
    use crate::induced_subgraph::IsInducedSubgraph;

    #[test]
    fn locate() {
        let tile = Coords::new(0, 0);
        let tile_right = tile.right().unwrap();

        let mut map = IndoorsMap::new_empty();
        map.0.insert(tile);

        assert!((&map).to_grid(&tile).is_some());
        assert!((&map).to_grid(&tile_right).is_none());

        map.0.insert(tile_right);

        assert!((&map).to_grid(&tile_right).is_some());
    }

    #[test]
    fn grid_trait_traversal() {
        let tile = Coords::new(0, 0);
        let tile_right = tile.right().unwrap();

        let mut map = IndoorsMap::new_empty();
        map.0.insert(tile);
        map.0.insert(tile_right);

        assert_eq!((&map).to_grid(&tile).right().left().unwrap().tile, tile);
        assert!((&map).to_grid(&tile).right().right().is_none());
    }
}
