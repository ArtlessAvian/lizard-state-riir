use std::collections::HashSet;

use crate::coords::Coords;
use crate::coords::PlanarProjection;
use crate::grid::Direction;
use crate::grid::Grid;

/// Implementors are runtime defined induced subgraphs of the tiles. Tiles are still connected as they defined.
///
/// Trait implementors can use `SubsetElement` to traverse the subset, since it implements `Grid`.
pub trait IsInducedSubgraph {
    type Original: Grid<Neighbor = Self::Original>;

    fn to_grid<'a>(&'a self, tile: &Self::Original) -> Option<InducedSubgraphElement<'a, Self>>;
}

#[derive(Debug, Hash)]
pub struct InducedSubgraphElement<'a, Subgraph>
where
    Subgraph: IsInducedSubgraph + ?Sized,
{
    pub map: &'a Subgraph,
    pub tile: &'a Subgraph::Original,
}

impl<Subgraph> Grid for InducedSubgraphElement<'_, Subgraph>
where
    Subgraph: IsInducedSubgraph + ?Sized,
{
    type Neighbor = Self;

    fn go(&self, dir: Direction) -> Option<Self::Neighbor> {
        self.map.to_grid(&self.tile.go(dir)?)
    }
}

impl<Subset> PlanarProjection for InducedSubgraphElement<'_, Subset>
where
    Subset: IsInducedSubgraph + ?Sized,
    Subset::Original: PlanarProjection,
{
    fn project_coords(&self) -> Coords {
        self.tile.project_coords()
    }
}

/// A set of tiles, connected if they exist in the set.
#[derive(Debug)]
pub struct IndoorsMap(HashSet<Coords>);

impl IndoorsMap {
    fn new_empty() -> Self {
        Self(HashSet::new())
    }
}

impl IsInducedSubgraph for IndoorsMap {
    type Original = Coords;

    fn to_grid<'a>(&'a self, tile: &Self::Original) -> Option<InducedSubgraphElement<'a, Self>> {
        self.0
            .get(tile)
            .map(|tile| InducedSubgraphElement { map: self, tile })
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

        assert!(map.to_grid(&tile).is_some());
        assert!(map.to_grid(&tile_right).is_none());

        map.0.insert(tile_right);

        assert!(map.to_grid(&tile_right).is_some());
    }

    #[test]
    fn grid_trait_traversal() {
        let tile = Coords::new(0, 0);
        let tile_right = tile.right().unwrap();

        let mut map = IndoorsMap::new_empty();
        map.0.insert(tile);
        map.0.insert(tile_right);

        assert_eq!(map.to_grid(&tile).right().left().unwrap().tile, &tile);
        assert!(map.to_grid(&tile).right().right().is_none());
    }
}
