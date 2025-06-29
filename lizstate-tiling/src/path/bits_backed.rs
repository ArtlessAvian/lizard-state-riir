use crate::path::BoundedPathLike;
use crate::path::Direction;

/// Pairs of bits interpreted as `Directions`.
///
/// The implementation *does* shift every element, compared to the array version.
/// This does let us derive Eq.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct PathBitString(u8, [u8; 7]);

impl PathBitString {
    /*
        // Suppose we want to store N elements.
        // We need 2N storage bits, and ceil(log_2(N)) bits to store size.
        // When N = 14, we use 4 len bits and 28 storage bits.
        pub const MAX_CAPACITY: usize = 29;
        const SIZE_BITS: usize = 6;

        const _MAX_CAP_REPRESENTABLE: () = assert!((1 << Self::SIZE_BITS) - 1 < Self::MAX_CAPACITY);
        const _FITS_IN_U32: () = assert!(Self::MAX_CAPACITY * 2 + Self::SIZE_BITS <= 64);
    */

    // We don't need to be *that* efficient.
    // We barely lose capacity.
    const MAX_CAPACITY: usize = 7 * 8 / 2;

    pub const fn new() -> Self {
        PathBitString(0, [0; 7])
    }

    pub const fn debug_assert_valid(self) -> Self {
        debug_assert!({
            let mask = u64::MAX << (2 * self.0);
            let masked = Self::reinterpret(self.1) & mask;
            masked == 0
        });
        self
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0 as usize
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[expect(clippy::many_single_char_names, reason = "silly const destructuring")]
    #[must_use]
    pub const fn reinterpret(bytes: [u8; 7]) -> u64 {
        let [a, b, c, d, e, f, g] = bytes;
        u64::from_le_bytes([a, b, c, d, e, f, g, 0])
    }

    #[expect(clippy::many_single_char_names, reason = "silly const destructuring")]
    #[must_use]
    pub const fn rereinterpret(val: u64) -> [u8; 7] {
        let [a, b, c, d, e, f, g, _] = val.to_le_bytes();
        [a, b, c, d, e, f, g]
    }

    #[must_use]
    pub const fn push(&self, dir: Direction) -> Option<Self> {
        if self.len() >= Self::MAX_CAPACITY {
            None
        } else {
            let raw = Self::reinterpret(self.1);
            let new_raw = (raw << 2) + Self::dir_to_pattern(dir);
            let new = Self(self.0 + 1, Self::rereinterpret(new_raw)).debug_assert_valid();
            Some(new)
        }
    }

    /// # Errors
    /// The path is already empty.
    pub const fn pop(&self) -> Result<(Self, Direction), super::PathAlreadyEmpty> {
        if self.is_empty() {
            Err(super::PathAlreadyEmpty)
        } else {
            let raw = Self::reinterpret(self.1);
            let new_raw = raw >> 2;
            let new = Self(self.0 - 1, Self::rereinterpret(new_raw)).debug_assert_valid();
            let dir = Self::pattern_to_dir(raw & 0b11);
            Ok((new, dir))
        }
    }

    #[expect(clippy::missing_panics_doc, reason = "expect")]
    pub const fn inverse(&self) -> Self {
        // i could write unreadable bit level shenanigans, but lets do this the normal way.
        let mut out = Self::new();
        let mut copy = *self;
        while let Ok((next, dir)) = copy.pop() {
            copy = next;
            out = out
                .push(dir.inverse())
                .expect("we cannot pop more than MAX_CAPACITY elements before the loop terminates");
        }
        out.debug_assert_valid()
    }

    #[must_use]
    pub const fn dir_to_pattern(dir: Direction) -> u64 {
        // Patterns chosen so that reversing the bits and flipping both if same results in the inverse.
        match dir {
            Direction::Up => 0b00,
            Direction::Down => 0b11,
            Direction::Right => 0b01,
            Direction::Left => 0b10,
        }
    }

    pub const fn pattern_to_dir(two_bits: u64) -> Direction {
        match two_bits {
            0b00 => Direction::Up,
            0b11 => Direction::Down,
            0b01 => Direction::Right,
            0b10 => Direction::Left,
            _ => unreachable!(),
        }
    }
}

impl Default for PathBitString {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for PathBitString {
    type Item = Direction;
    type IntoIter = PathBitStringIter;

    fn into_iter(self) -> Self::IntoIter {
        PathBitStringIter { string: self, i: 0 }
    }
}

pub struct PathBitStringIter {
    string: PathBitString,
    i: u8,
}

impl Iterator for PathBitStringIter {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.string.0 {
            None
        } else {
            let raw = PathBitString::reinterpret(self.string.1);
            let pat = (raw >> (2 * (self.string.0 - 1 - self.i))) & 0b11;
            let dir = PathBitString::pattern_to_dir(pat);

            self.i += 1;
            Some(dir)
        }
    }
}

impl BoundedPathLike for PathBitString {
    const MAX_CAPACITY: usize = 7 * 8 / 2;

    fn push(&self, dir: Direction) -> Option<Self> {
        self.push(dir)
    }

    fn pop(&self) -> Result<(Self, Direction), super::PathAlreadyEmpty> {
        self.pop()
    }
}

#[cfg(test)]
mod tests {
    use crate::direction::Direction;
    use crate::path::bits_backed::PathBitString;

    #[test]
    fn basic_inverses() {
        let right = PathBitString::new().push(Direction::Right).unwrap();
        let left = PathBitString::new().push(Direction::Left).unwrap();
        assert_eq!(right.inverse(), left);
        assert_eq!(left.inverse(), right);

        let up = PathBitString::new().push(Direction::Up).unwrap();
        let down = PathBitString::new().push(Direction::Down).unwrap();
        assert_eq!(up.inverse(), down);
        assert_eq!(down.inverse(), up);
    }

    #[test]
    fn pair_inverses() {
        let right_up = PathBitString::new()
            .push(Direction::Right)
            .unwrap()
            .push(Direction::Up)
            .unwrap();
        let down_left = PathBitString::new()
            .push(Direction::Down)
            .unwrap()
            .push(Direction::Left)
            .unwrap();
        assert_eq!(right_up.inverse(), down_left);
        assert_eq!(down_left.inverse(), right_up);
    }

    #[test]
    fn iter() {
        let right_up = PathBitString::new()
            .push(Direction::Right)
            .unwrap()
            .push(Direction::Up)
            .unwrap();

        let mut iter = right_up.into_iter();
        assert_eq!(iter.next(), Some(Direction::Right));
        assert_eq!(iter.next(), Some(Direction::Up));
        assert_eq!(iter.next(), None);
    }
}
