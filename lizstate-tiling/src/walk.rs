//! A sequence of edges, described by directions.
//! Without knowing the graph we are walking on, we cannot know if a walk is a path (no repeated vertices)
//! We can *sort of* know if a walk is a trail (no repeated edges).

#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "type can grow maybe (and still be copy)"
)]

use core::num::NonZero;
use core::num::NonZeroU64;

use crate::direction::Direction;

#[derive(Debug)]
pub struct WalkIsEmpty;
pub struct WalkIsFull;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct Walk {
    // A number to be interpreted in 4-ary.
    // The leading digit is ignored
    // The rest of the digits are treated as directions.
    encoding: NonZeroU64,
}

impl Walk {
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
            let encoding = NonZero::new(encoding).expect("nonoverflowing shl is never zero");
            Ok(Self { encoding })
        } else {
            Err(WalkIsFull)
        }
    }

    /// # Errors
    /// Nothing to pop from walk.
    pub const fn pop_copy(&self) -> Result<(Self, Direction), WalkIsEmpty> {
        let shr = self.encoding.get() >> 2;
        let maybe_encoding = NonZero::new(shr >> 2);
        if let Some(encoding) = maybe_encoding {
            let pattern = self.encoding.get() & 0b11;
            Ok((Self { encoding }, Self::pattern_to_dir(pattern)))
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
