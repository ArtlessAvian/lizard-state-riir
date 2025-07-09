#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "type can grow maybe (and still be copy)"
)]

use core::num::NonZeroU64;

use crate::direction::Direction;
use crate::walk::WalkIsEmpty;
use crate::walk::WalkIsFull;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkPartial;
use crate::walk::traits::IsAWalkRaw;

/// Enumeration in the math sense.
///
/// Highly efficient storage!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct WalkEnum {
    // A number to be interpreted in 4-ary.
    // The leading digit is ignored
    // The rest of the digits are treated as directions.
    encoding: NonZeroU64,
}

impl WalkEnum {
    pub const fn new_empty() -> Self {
        Self {
            encoding: NonZeroU64::MIN,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.encoding.get() >> 2 == 0
    }

    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.encoding.get().checked_shl(2).is_none()
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        // Integer division intentional.
        // 1..4 -> 0 (we want to ignore the leading bit)
        // 4..16 -> 1
        // 16..64 -> 2
        (self.encoding.ilog2() / 2) as usize
    }

    /// # Errors
    /// Extended walk cannot be stored in Storage.
    /// # Panics
    /// Unwrapping `Option<NonZero>`, but it should never happen.
    pub const fn push_copy(&self, dir: Direction) -> Result<Self, WalkIsFull> {
        let checked_shl = self.encoding.get().checked_shl(2);
        if let Some(shifted) = checked_shl {
            let encoding = shifted | Self::dir_to_pattern(dir);
            let encoding = NonZeroU64::new(encoding).expect("nonoverflowing shl is never zero");
            Ok(Self { encoding })
        } else {
            Err(WalkIsFull)
        }
    }

    // The direction at the end of the walk.
    /// # Errors
    /// Nothing to pop from walk.
    pub const fn peek(&self) -> Result<Direction, WalkIsEmpty> {
        if self.is_empty() {
            Err(WalkIsEmpty)
        } else {
            Ok(Self::pattern_to_dir(self.encoding.get() & 0b11))
        }
    }

    /// # Errors
    /// Nothing to pop from walk.
    pub const fn pop_copy(&self) -> Result<(Self, Direction), WalkIsEmpty> {
        let maybe_encoding = NonZeroU64::new(self.encoding.get() >> 2);
        let maybe_peek = self.peek();

        if let Some(encoding) = maybe_encoding
            && let Ok(peek) = maybe_peek
        {
            Ok((Self { encoding }, peek))
        } else {
            Err(WalkIsEmpty)
        }
    }

    const fn dir_to_pattern(dir: Direction) -> u64 {
        // Patterns chosen so that reversing the bits and flipping both if same results in the inverse.
        match dir {
            Direction::Up => 0b00,
            Direction::Down => 0b11,
            Direction::Right => 0b01,
            Direction::Left => 0b10,
        }
    }

    const fn pattern_to_dir(two_bits: u64) -> Direction {
        match two_bits {
            0b00 => Direction::Up,
            0b11 => Direction::Down,
            0b01 => Direction::Right,
            0b10 => Direction::Left,
            _ => unreachable!(),
        }
    }
}

impl IsAWalkPartial for WalkEnum {
    type PushError = WalkIsFull;

    fn new_empty() -> Self {
        Self::new_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn peek(&self) -> Result<Direction, WalkIsEmpty> {
        self.peek()
    }

    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError> {
        *self = self.push_copy(dir)?;
        Ok(())
    }

    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty> {
        let popped;
        (*self, popped) = self.pop_copy()?;
        Ok(popped)
    }

    fn push_copy(&self, dir: Direction) -> Result<Self, Self::PushError> {
        self.push_copy(dir)
    }

    fn pop_copy(&self) -> Result<(Self, Direction), WalkIsEmpty> {
        self.pop_copy()
    }
}

impl IntoIterator for WalkEnum {
    type Item = Direction;
    type IntoIter = WalkEnumIter;

    fn into_iter(self) -> Self::IntoIter {
        // HACK: We want a queue, but we have a stack interface.
        // Inverting both reverses the sequence and inverts each direction.
        // We'll need to revert the directions later.
        let mut inverse = self;
        inverse.inverse_mut();
        WalkEnumIter { inverse }
    }
}

pub struct WalkEnumIter {
    inverse: WalkEnum,
}

impl Iterator for WalkEnumIter {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        // HACK: We can pop directions the inverse of our path. We can revert individual directions now.
        self.inverse.pop_mut().ok().map(Direction::inverse)
    }
}

impl IsAWalk for WalkEnum {}

impl IsAWalkRaw for WalkEnum {}
