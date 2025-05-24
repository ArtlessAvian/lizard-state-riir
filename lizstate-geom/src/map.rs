use std::collections::HashMap;
use std::hash::Hash;

use crate::tiles::Direction;
use crate::tiles::Grid;
use crate::tiles::PlanarProjection;

#[derive(Debug)]
#[must_use]
pub struct TileMap<Tile, Decoration = ()> {
    tiles: HashMap<Tile, Decoration>,
}

impl<T, D> TileMap<T, D> {
    fn new_empty() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }
}

impl<T, D> TileMap<T, D>
where
    T: Eq + Hash,
{
    fn locate(&self, tile: &T) -> Option<InTilemap<'_, T, D>> {
        self.tiles
            .get_key_value(tile)
            .map(|(tile, _)| InTilemap { map: self, tile })
    }
}

#[derive(Debug)]
#[must_use]
pub struct InTilemap<'a, T, D> {
    map: &'a TileMap<T, D>,
    tile: &'a T,
}

impl<T, D> Grid for InTilemap<'_, T, D>
where
    T: Eq + Hash,
    T: Grid<NeighborType = T>,
{
    type NeighborType = Self;

    fn go(&self, dir: Direction) -> Option<Self::NeighborType> {
        self.map.locate(&self.tile.go(dir)?)
    }
}

impl<T, D> PlanarProjection for InTilemap<'_, T, D>
where
    T: Eq + Hash,
    T: Grid<NeighborType = T> + PlanarProjection,
{
    fn project_coords(&self) -> (i32, i32) {
        self.tile.project_coords()
    }
}

#[cfg(test)]
mod tests {
    use super::TileMap;
    use crate::tiles::Grid;
    use crate::tiles::Tile;

    #[test]
    fn locate() {
        let tile = Tile::new(0, 0);
        let tile_right = tile.right().unwrap();

        let mut map = TileMap::new_empty();
        map.tiles.insert(tile, ());

        assert!(map.locate(&tile).is_some());
        assert!(map.locate(&tile_right).is_none());

        map.tiles.insert(tile_right, ());

        assert!(map.locate(&tile_right).is_some());
    }

    #[test]
    fn grid_trait_traversal() {
        let tile = Tile::new(0, 0);
        let tile_right = tile.right().unwrap();

        let mut map = TileMap::new_empty();
        map.tiles.insert(tile, ());
        map.tiles.insert(tile_right, ());

        assert_eq!(map.locate(&tile).right().left().unwrap().tile, &tile);
        assert!(map.locate(&tile).right().right().is_none());
    }
}
