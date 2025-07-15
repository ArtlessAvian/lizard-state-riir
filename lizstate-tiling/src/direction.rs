use lizstate_sequence_enumeration::IsSequenceable;

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

impl IsSequenceable for Direction {
    const MAX_EXCLUSIVE: u64 = 4;

    fn to_value(self) -> u64 {
        match self {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::Right => 2,
            Direction::Left => 3,
        }
    }

    fn from_value(value: u64) -> Self {
        match value {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Right,
            3 => Direction::Left,
            Self::MAX_EXCLUSIVE.. => panic!(),
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

impl IsSequenceable for Nonbackwards {
    const MAX_EXCLUSIVE: u64 = 3;

    fn to_value(self) -> u64 {
        match self {
            Nonbackwards::TurnLeft => 0,
            Nonbackwards::GoStraight => 1,
            Nonbackwards::TurnRight => 2,
        }
    }

    fn from_value(value: u64) -> Self {
        match value {
            0 => Nonbackwards::TurnLeft,
            1 => Nonbackwards::GoStraight,
            2 => Nonbackwards::TurnRight,
            _ => panic!(),
        }
    }
}
