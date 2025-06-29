use crate::direction::Direction;
use crate::tiling::HasSquareTiling;
use crate::tiling::Tile;
use crate::tiling::TileUndirectedEdge;

impl<TileType> Tile for TileUndirectedEdge<TileType> where TileType: Tile {}

/// For every square in a tiling, replace every square's edge with a diamond.
///
/// Alternatively, replace every graph edge with a vertex.
/// Directions are rotated by 45 degrees.
#[derive(PartialEq, Eq)]
pub struct EdgeTiling<Tiling>(Tiling);

impl<Tiling, TileType> HasSquareTiling<TileUndirectedEdge<TileType>> for &EdgeTiling<Tiling>
where
    Tiling: HasSquareTiling<TileType>,
    TileType: Tile,
{
    fn get_origin(&self) -> TileUndirectedEdge<TileType> {
        let origin = self.0.get_origin();
        for dir in [
            Direction::Up,
            Direction::Down,
            Direction::Right,
            Direction::Left,
        ] {
            if let Some(edge) = TileUndirectedEdge::new(&self.0, &origin, dir) {
                return edge;
            }
        }
        panic!("origin is disconnected?")
    }

    fn step(
        &self,
        tile: &TileUndirectedEdge<TileType>,
        dir: Direction,
    ) -> Option<TileUndirectedEdge<TileType>> {
        match tile {
            // A horizontal edge's neighbor will always be vertical.
            //      O   O
            // LEFT |   | UP
            //      O - O
            // DOWN |   | RIGHT
            //      O   O
            //
            TileUndirectedEdge::Horizontal { left, right } => match dir {
                Direction::Up => TileUndirectedEdge::new(&self.0, right, Direction::Up),
                Direction::Down => TileUndirectedEdge::new(&self.0, left, Direction::Down),
                Direction::Right => TileUndirectedEdge::new(&self.0, right, Direction::Down),
                Direction::Left => TileUndirectedEdge::new(&self.0, left, Direction::Up),
            },
            // A vertical edge's neighbor will always be horizontal.
            //   L   U
            // O - O - O
            //     |
            // O - O - O
            //   D   R
            //
            TileUndirectedEdge::Vertical { down, up } => match dir {
                Direction::Up => TileUndirectedEdge::new(&self.0, up, Direction::Right),
                Direction::Down => TileUndirectedEdge::new(&self.0, down, Direction::Left),
                Direction::Right => TileUndirectedEdge::new(&self.0, down, Direction::Right),
                Direction::Left => TileUndirectedEdge::new(&self.0, up, Direction::Left),
            },
        }
    }
}
