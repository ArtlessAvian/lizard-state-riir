use crate::direction::Direction;
use crate::tiling_graph::IsASpace;
use crate::tiling_graph::IsATile;
use crate::tiling_graph::IsTilingGraph;

/// A space replacing every tile from the `Space` generic
/// with 16x16 `ExpandedTile`s.
struct ExpandedSpace<Space>(Space);

impl<Space: IsASpace> IsASpace for ExpandedSpace<Space> {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct ExpandedTile<Tile> {
    chunk: Tile,
    x: u8,
    y: u8,
}

impl<Tile: IsATile> IsATile for ExpandedTile<Tile> {}

impl<Space: IsTilingGraph> IsTilingGraph for ExpandedSpace<Space> {
    type Tile = ExpandedTile<Space::Tile>;

    fn get_origin(&self) -> Self::Tile {
        ExpandedTile {
            chunk: self.0.get_origin(),
            x: 0,
            y: 0,
        }
    }

    fn step(
        &self,
        tile: &Self::Tile,
        dir: crate::direction::Direction,
    ) -> Result<Self::Tile, crate::tiling_graph::StepError> {
        let chunk = if match dir {
            Direction::Up => tile.y == 15,
            Direction::Down => tile.y == 0,
            Direction::Right => tile.x == 15,
            Direction::Left => tile.x == 0,
        } {
            self.0.step(&tile.chunk, dir)?
        } else {
            tile.chunk
        };

        let x = match dir {
            Direction::Up | Direction::Down => tile.x,
            Direction::Right => (tile.x + 1) % 16,
            Direction::Left => (tile.x + 16 - 1) % 16,
        };

        let y = match dir {
            Direction::Up => (tile.y + 1) % 16,
            Direction::Down => (tile.y + 16 - 1) % 16,
            Direction::Right | Direction::Left => tile.y,
        };

        Ok(ExpandedTile { chunk, x, y })
    }
}

#[cfg(test)]
mod tests {
    use crate::coords::CartesianCoords;
    use crate::direction::Direction;
    use crate::euclidean_plane::TheEuclideanPlane;
    use crate::expansion::ExpandedSpace;
    use crate::expansion::ExpandedTile;
    use crate::tiling_graph::IsTilingGraph;

    #[test]
    fn step_within() {
        let space = ExpandedSpace::<TheEuclideanPlane>(TheEuclideanPlane);

        let origin = space.get_origin();
        assert_eq!(
            origin,
            ExpandedTile {
                chunk: CartesianCoords { x: 0, y: 0 },
                x: 0,
                y: 0,
            }
        );

        let step = space.step(&origin, Direction::Right).unwrap();
        assert_eq!(step.chunk, origin.chunk);

        let step = space.step(&step, Direction::Up).unwrap();
        assert_eq!(step.chunk, origin.chunk);

        let step = space.step(&step, Direction::Left).unwrap();
        assert_eq!(step.chunk, origin.chunk);

        let step = space.step(&step, Direction::Down).unwrap();
        assert_eq!(step.chunk, origin.chunk);
    }

    #[test]
    fn step_through() {
        let space = ExpandedSpace::<TheEuclideanPlane>(TheEuclideanPlane);

        let origin = space.get_origin();

        let step = space.step(&origin, Direction::Left).unwrap();
        assert_eq!(step.chunk, CartesianCoords { x: -1, y: 0 });

        let step = space.step(&step, Direction::Down).unwrap();
        assert_eq!(step.chunk, CartesianCoords { x: -1, y: -1 });

        let step = space.step(&step, Direction::Right).unwrap();
        assert_eq!(step.chunk, CartesianCoords { x: 0, y: -1 });

        let step = space.step(&step, Direction::Up).unwrap();
        assert_eq!(step.chunk, CartesianCoords { x: 0, y: 0 });
    }
}
