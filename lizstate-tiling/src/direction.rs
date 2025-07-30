use lizstate_sequence_enumeration::digit::Digit;
use lizstate_sequence_enumeration::digit::IsSmallEnum;

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

impl IsSmallEnum for Direction {
    type Digit = Digit<4>;

    fn to_digit(&self) -> Self::Digit {
        match self {
            Direction::Up => Digit::from_modulo(0),
            Direction::Down => Digit::from_modulo(1),
            Direction::Right => Digit::from_modulo(2),
            Direction::Left => Digit::from_modulo(3),
        }
    }

    fn from_digit(digit: Self::Digit) -> Self {
        match digit.get() {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Right,
            3 => Direction::Left,
            4.. => unreachable!(),
        }
    }
}
