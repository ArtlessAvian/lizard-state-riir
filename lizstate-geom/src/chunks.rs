use crate::grid::Direction;
use crate::grid::Grid;

const CHUNK_SIZE: i8 = 8;

#[derive(Debug, PartialEq, Eq)]
struct Chunk<T> {
    tile: T,
    x: i8,
    y: i8,
}

impl<T> Grid for Chunk<T>
where
    T: Grid<Neighbor = T> + Clone,
{
    type Neighbor = Self;

    fn go(&self, dir: crate::grid::Direction) -> Option<Self::Neighbor> {
        const LAST_VALID_COORD: i8 = CHUNK_SIZE - 1;

        let tile = match (dir, self.x, self.y) {
            (Direction::Up, _, LAST_VALID_COORD)
            | (Direction::Down, _, 0)
            | (Direction::Left, 0, _)
            | (Direction::Right, LAST_VALID_COORD, _) => self.tile.go(dir)?,
            _ => self.tile.clone(),
        };

        let x = (self.x
            + match dir {
                Direction::Left => -1,
                Direction::Right => 1,
                _ => 0,
            })
        .rem_euclid(CHUNK_SIZE);

        let y = (self.y
            + match dir {
                Direction::Up => 1,
                Direction::Down => -1,
                _ => 0,
            })
        .rem_euclid(CHUNK_SIZE);

        Some(Chunk { tile, x, y })
    }
}
