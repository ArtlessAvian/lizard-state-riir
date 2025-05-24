use std::collections::HashSet;
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

#[derive(Debug, Hash)]
pub struct InducedSubgraphElement<Subgraph>
where
    Subgraph: IsInducedSubgraph,
{
    pub map: Subgraph,
    pub tile: Subgraph::Original,
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
pub struct IndoorsMap(Rc<HashSet<Coords>>);

impl IndoorsMap {
    fn new_empty() -> Self {
        Self(Rc::new(HashSet::new()))
    }
}

impl IsInducedSubgraph for IndoorsMap {
    type Original = Coords;

    fn to_grid(&self, tile: &Self::Original) -> Option<InducedSubgraphElement<Self>> {
        self.0.get(tile).map(|tile| InducedSubgraphElement {
            map: self.clone(),
            tile: *tile,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::IndoorsMap;
    use crate::coords::Coords;
    use crate::grid::GridShortcuts;
    use crate::induced_subgraph::IsInducedSubgraph;

    #[test]
    fn locate() {
        let tile = Coords::new(0, 0);
        let tile_right = tile.right().unwrap();

        let mut map = IndoorsMap::new_empty();
        Rc::get_mut(&mut map.0).unwrap().insert(tile);

        assert!(map.to_grid(&tile).is_some());
        assert!(map.to_grid(&tile_right).is_none());

        Rc::get_mut(&mut map.0).unwrap().insert(tile_right);

        assert!(map.to_grid(&tile_right).is_some());
    }

    #[test]
    fn grid_trait_traversal() {
        let tile = Coords::new(0, 0);
        let tile_right = tile.right().unwrap();

        let mut map = IndoorsMap::new_empty();
        Rc::get_mut(&mut map.0).unwrap().insert(tile);
        Rc::get_mut(&mut map.0).unwrap().insert(tile_right);

        assert_eq!(map.to_grid(&tile).right().left().unwrap().tile, tile);
        assert!(map.to_grid(&tile).right().right().is_none());
    }
}
