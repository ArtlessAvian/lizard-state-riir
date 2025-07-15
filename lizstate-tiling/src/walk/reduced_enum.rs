use core::num::NonZeroU64;

use crate::direction::Direction;

/// Steps relative to a previous Direction, excluding Backwards.
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq)]
enum RelativeDirection {
    MyLeft,
    Forward,
    MyRight,
}

#[must_use]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ReducedEnum {
    NonEmpty(NonEmpty),
    Empty,
}

impl ReducedEnum {
    fn new_empty() -> Self {
        Self::Empty
    }

    fn new_with_initial(dir: Direction) -> Self {
        Self::NonEmpty(NonEmpty::new_with_initial(dir))
    }

    fn is_empty(self) -> bool {
        self == Self::Empty
    }

    fn len(self) -> usize {
        match self {
            Self::NonEmpty(nonempty) => nonempty.len(),
            Self::Empty => 0,
        }
    }
}

impl From<ReducedEnum> for Option<NonEmpty> {
    fn from(val: ReducedEnum) -> Self {
        match val {
            ReducedEnum::NonEmpty(nonempty) => Some(nonempty),
            ReducedEnum::Empty => None,
        }
    }
}

impl From<NonEmpty> for ReducedEnum {
    fn from(val: NonEmpty) -> Self {
        ReducedEnum::NonEmpty(val)
    }
}

/// An initial direction, and a sequence of nonbackwards steps.
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq)]
struct NonEmpty(NonZeroU64);

impl NonEmpty {
    fn new_with_initial(dir: Direction) -> Self {
        Self(Self::dir_to_nonzero(dir))
    }

    fn new_with_both(dir: Direction, seq: RelativeSequence) -> Option<Self> {
        let dingus = Self::dir_to_nonzero(dir).checked_add(seq.0)?;
        Some(Self(dingus))
    }

    fn get_initial_direction(self) -> Direction {
        //   (NonEmptyReducedEnum - 1) % 4
        // = ((4 * RotationSequence + Direction + 1) - 1) % 4
        // = ((Direction + 1) - 1) % 4
        // = Direction % 4
        let quaternary = (self.0.get() - 1) % 4;
        NonEmpty::quaternary_to_dir(quaternary)
    }

    fn to_nonbackwards(self) -> RelativeSequence {
        //   (NonEmptyReducedEnum - 1) / 4
        // = ((4 * RotationSequence + Direction + 1) - 1) / 4
        // = 4 * RotationSequence / 4 + Direction / 4
        // = RotationSequence + 0
        RelativeSequence((self.0.get() - 1) / 4)
    }

    fn len(self) -> usize {
        1 + self.to_nonbackwards().len()
    }

    const fn dir_to_quaternary(dir: Direction) -> u64 {
        match dir {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::Right => 2,
            Direction::Left => 3,
        }
    }

    const fn dir_to_nonzero(dir: Direction) -> NonZeroU64 {
        NonZeroU64::new(Self::dir_to_quaternary(dir)).unwrap()
    }

    const fn nonzero_to_dir(nonzero: NonZeroU64) -> Direction {
        Self::quaternary_to_dir(nonzero.get() - 1)
    }

    const fn quaternary_to_dir(two_bits: u64) -> Direction {
        match two_bits {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Right,
            3 => Direction::Left,
            _ => unreachable!(),
        }
    }
}

/// EXPLICITLY PRIVATE.
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq)]
struct RelativeSequence(u64);

impl RelativeSequence {
    fn new_empty() -> RelativeSequence {
        RelativeSequence(0)
    }

    fn len(self) -> usize {
        if self.0 == 0 {
            0
        } else {
            1 + self.pop_or_noop().len()
        }
    }

    fn push(self, rot: RelativeDirection) -> Self {
        Self(self.0 * 3 + Self::rot_to_ternary(rot) + 1)
    }

    fn pop_or_noop(self) -> Self {
        Self((self.0 - 1) / 3)
    }

    fn rot_to_ternary(rot: RelativeDirection) -> u64 {
        match rot {
            RelativeDirection::MyLeft => 0,
            RelativeDirection::Forward => 1,
            RelativeDirection::MyRight => 2,
        }
    }
}
