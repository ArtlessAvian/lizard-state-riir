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

/// Rotation to apply to a Direction.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rotation {
    TurnLeft,
    GoStraight,
    TurnRight,
    DoAUTurn,
}

impl Rotation {
    pub const fn apply_to_direction(self, dir: Direction) -> Direction {
        match self {
            Rotation::GoStraight => dir,
            Rotation::DoAUTurn => dir.inverse(),
            Rotation::TurnLeft => match dir {
                Direction::Up => Direction::Left,
                Direction::Down => Direction::Right,
                Direction::Right => Direction::Up,
                Direction::Left => Direction::Down,
            },
            Rotation::TurnRight => match dir {
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
                Direction::Right => Direction::Down,
                Direction::Left => Direction::Up,
            },
        }
    }

    pub const fn from_pair(before: Direction, after: Direction) -> Rotation {
        match (before, after) {
            (Direction::Up, Direction::Up)
            | (Direction::Down, Direction::Down)
            | (Direction::Right, Direction::Right)
            | (Direction::Left, Direction::Left) => Rotation::GoStraight,
            (Direction::Up, Direction::Down)
            | (Direction::Down, Direction::Up)
            | (Direction::Right, Direction::Left)
            | (Direction::Left, Direction::Right) => Rotation::DoAUTurn,
            (Direction::Up, Direction::Right)
            | (Direction::Down, Direction::Left)
            | (Direction::Right, Direction::Down)
            | (Direction::Left, Direction::Up) => Rotation::TurnRight,
            (Direction::Up, Direction::Left)
            | (Direction::Down, Direction::Right)
            | (Direction::Right, Direction::Up)
            | (Direction::Left, Direction::Down) => Rotation::TurnLeft,
        }
    }
}

/// Non-uturn rotations.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nonbackwards {
    TurnLeft,
    GoStraight,
    TurnRight,
}

impl From<Nonbackwards> for Rotation {
    fn from(value: Nonbackwards) -> Self {
        match value {
            Nonbackwards::TurnLeft => Rotation::TurnLeft,
            Nonbackwards::GoStraight => Rotation::GoStraight,
            Nonbackwards::TurnRight => Rotation::TurnRight,
        }
    }
}

impl TryFrom<Rotation> for Nonbackwards {
    type Error = ();

    fn try_from(value: Rotation) -> Result<Self, Self::Error> {
        match value {
            Rotation::TurnLeft => Ok(Nonbackwards::TurnLeft),
            Rotation::GoStraight => Ok(Nonbackwards::GoStraight),
            Rotation::TurnRight => Ok(Nonbackwards::TurnRight),
            Rotation::DoAUTurn => Err(()),
        }
    }
}

impl IsSmallEnum for Nonbackwards {
    type Digit = Digit<3>;

    fn to_digit(&self) -> Self::Digit {
        match self {
            Nonbackwards::TurnLeft => Digit::from_modulo(0),
            Nonbackwards::GoStraight => Digit::from_modulo(1),
            Nonbackwards::TurnRight => Digit::from_modulo(2),
        }
    }

    fn from_digit(digit: Self::Digit) -> Self {
        match digit.get() {
            0 => Nonbackwards::TurnLeft,
            1 => Nonbackwards::GoStraight,
            2 => Nonbackwards::TurnRight,
            _ => panic!(),
        }
    }
}
