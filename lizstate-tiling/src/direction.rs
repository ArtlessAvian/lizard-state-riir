#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    pub const fn inverse(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }

    #[must_use]
    pub const fn const_eq(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Up)
                | (Direction::Down, Direction::Down)
                | (Direction::Right, Direction::Right)
                | (Direction::Left, Direction::Left)
        )
    }
}
