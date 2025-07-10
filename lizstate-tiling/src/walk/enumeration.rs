#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "type can grow maybe (and still be copy)"
)]

use crate::direction::Direction;
use crate::walk::WalkIsEmpty;
use crate::walk::WalkIsFull;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkPartial;
use crate::walk::traits::IsAWalkRaw;

/// Enumeration in the math sense.
///
/// This is *like* how binary trees are indexed.
/// A parent i has children 2i+1 and 2i+2.
/// We have a quatenary (4-ary) tree instead, with children ranging from (4i+1)..=(4i+4).
/// This is a bijection between natural numbers and nodes!
///
/// With this approach, the greatest index path of length h is
/// f(h) = -1 + \sum_{n=0}^{h} 4^h
/// Notably f(31) <= `u64::MAX` < f(32),
/// so a `u64` can enumerate all walks up to length 31, with a lot of room to spare.
///
/// About the "wasted" space, we can use `NonZero` and shift everything,
/// but it makes the math sad. (children range from (4i-2)..=(4i+1) )
/// We *can* use accursed attributes:
/// `#[rustc_layout_scalar_valid_range_end()]`
/// But that might be overengineering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct WalkEnum(u64);

impl WalkEnum {
    const SMALLEST_OF_LEN: [u64; 32] = {
        let mut out = [0; 32];
        let mut len = 1;
        while len < 32 {
            out[len] = out[len - 1] * 4 + 1;
            len += 1;
        }
        out
    };

    const LARGEST_OF_LEN: [u64; 32] = {
        let mut out = [0; 32];
        let mut len = 1;
        while len < 32 {
            out[len] = out[len - 1] * 4 + 4;
            len += 1;
        }
        out
    };

    const SMALLEST_FULL: u64 = Self::SMALLEST_OF_LEN[31];
    const LARGEST_FULL: u64 = Self::LARGEST_OF_LEN[31];

    pub const fn new_empty() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn try_from_index(index: u64) -> Option<Self> {
        if index <= Self::LARGEST_FULL {
            Some(Self(index))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.0 >= Self::SMALLEST_FULL
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        let mut len = 0;
        while len < 32 {
            if self.0 <= Self::LARGEST_OF_LEN[len] {
                return len;
            }
            len += 1;
        }
        unreachable!()
    }

    const fn dir_to_quaternary(dir: Direction) -> u8 {
        match dir {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::Right => 2,
            Direction::Left => 3,
        }
    }

    const fn quaternary_to_dir(two_bits: u8) -> Direction {
        match two_bits {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Right,
            3 => Direction::Left,
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
        if self.is_empty() {
            Err(WalkIsEmpty)
        } else {
            #[expect(clippy::cast_possible_truncation, reason = "k mod 4 < 4")]
            let quaternary: u8 = (self.0 - 1).rem_euclid(4) as u8;
            Ok(Self::quaternary_to_dir(quaternary))
        }
    }

    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError> {
        if self.0 >= WalkEnum::SMALLEST_FULL {
            Err(WalkIsFull)
        } else {
            self.0 *= 4;
            self.0 += 1 + (dir as u64);
            Ok(())
        }
    }

    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty> {
        if let Ok(popped) = self.peek() {
            // We want to map 4k+1 through 4k+4 to k.
            self.0 -= 1; // 4k+0 through 4k+3
            self.0 /= 4; // k
            Ok(popped)
        } else {
            Err(WalkIsEmpty)
        }
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

#[cfg(test)]
mod tests {
    use crate::walk::enumeration::WalkEnum;

    #[test]
    fn walk_enum_len() {
        let mut min_index = 0u64;
        let mut max_index = 0u64;
        let mut expected_len = 0usize;
        while expected_len < 31 {
            let min_enum = WalkEnum::try_from_index(min_index).unwrap();
            let max_enum = WalkEnum::try_from_index(max_index).unwrap();

            assert_eq!(min_enum.len(), expected_len, "index: {min_index}");
            assert_eq!(max_enum.len(), expected_len, "index: {max_index}");
            // and we can reason that everything in between is the expected len.

            expected_len += 1;
            min_index = min_index * 4 + 1;
            max_index = max_index * 4 + 4;
        }
    }
}
