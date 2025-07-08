use crate::direction::Direction;
use crate::tiling::HasSquareTiling;
use crate::tiling::IsASpace;
use crate::tiling::IsATile;
use crate::tiling::TileUndirectedEdge;

impl<Tile> IsATile for TileUndirectedEdge<Tile> where Tile: IsATile {}

/// For every square in a tiling, replace every square's edge with a diamond.
///
/// Alternatively, replace every graph edge with a vertex.
/// Directions are rotated by 45 degrees.
#[derive(PartialEq, Eq)]
pub struct EdgeTiling<Space>(Space);

impl<Space> IsASpace for &EdgeTiling<Space> where Space: IsASpace {}

impl<Space, Tile> HasSquareTiling<TileUndirectedEdge<Tile>> for &EdgeTiling<Space>
where
    Space: HasSquareTiling<Tile>,
    Tile: IsATile,
{
    fn get_origin(&self) -> TileUndirectedEdge<Tile> {
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
        tile: &TileUndirectedEdge<Tile>,
        dir: Direction,
    ) -> Option<TileUndirectedEdge<Tile>> {
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
