#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "type can grow maybe (and still be copy)"
)]

use crate::tiling::Direction;

pub trait PathLike
where
    Self: Copy + Eq,
{
    fn append(&self, dir: Direction) -> Option<Self>;

    fn pop(&self) -> Option<(Self, Direction)>;

    #[must_use]
    fn inverse(&self) -> Self;
}

/// A string of `Directions`.
///
/// A bit space wasteful, but its alright.
/// Mostly exists for comparison.
#[derive(Debug, Clone, Copy)]
#[must_use]
struct PathString(usize, [Direction; 15]);

// Const implementations. Bleh.
impl PathString {
    pub const fn new() -> Self {
        Self(0, [Direction::Up; 15])
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.len() >= 15
    }

    #[must_use]
    pub const fn last(&self) -> Option<Direction> {
        if self.is_empty() {
            None
        } else {
            Some(self.1[self.0 - 1])
        }
    }

    #[must_use]
    pub const fn append(&self, dir: Direction) -> Option<Self> {
        if self.is_full() {
            return None;
        }

        let mut out = *self;
        out.1[out.0] = dir;
        out.0 += 1;
        Some(out)
    }

    #[must_use]
    pub const fn pop(&self) -> Option<(Self, Direction)> {
        if let Some(last) = self.last() {
            Some((Self(self.0 - 1, self.1), last))
        } else {
            None
        }
    }

    pub const fn inverse(&self) -> Self {
        let mut array = [Direction::Up; 15];

        let mut to = 0;
        while to < self.0 {
            let from = self.0 - 1 - to;
            array[to] = self.1[from].inverse();
            to += 1;
        }

        PathString(self.0, array)
    }

    pub fn as_slice(&self) -> &[Direction] {
        &self.1[..self.0]
    }

    pub fn iter(&self) -> impl Iterator<Item = Direction> {
        self.into_iter()
    }
}

impl PathLike for PathString {
    // fn new_from_iter(mut iter: impl Iterator<Item = Direction>) -> Option<Self> {
    //     let head = iter.by_ref().take(15);
    //     let mut tail = iter;

    //     if let Some(_) = tail.next() {
    //         // Doesn't fit.
    //         return None;
    //     }

    //     let mut count = 0;
    //     let mut array = [Direction::Up; 15];

    //     let zipped = array.iter_mut().zip(head);
    //     for (slot, el) in zipped {
    //         *slot = el;
    //         count += 1;
    //     }

    //     Some(Self(count, array))
    // }

    fn append(&self, dir: Direction) -> Option<Self> {
        self.append(dir)
    }

    fn pop(&self) -> Option<(Self, Direction)> {
        self.pop()
    }

    fn inverse(&self) -> Self {
        self.inverse()
    }
}

impl Default for PathString {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for PathString {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0) && (self.as_slice() == other.as_slice())
    }
}

impl Eq for PathString {}

impl<'a> IntoIterator for &'a PathString {
    type Item = Direction;
    type IntoIter = core::iter::Copied<core::slice::Iter<'a, Direction>>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter().copied()
    }
}

#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct PathBitString(u64);

impl PathBitString {
    // Suppose we want to store N elements.
    // We need 2N storage bits, and ceil(log_2(N)) bits to store size.
    // When N = 14, we use 4 len bits and 28 storage bits.
    pub const MAX_CAPACITY: usize = 29;
    const SIZE_BITS: usize = 6;

    const _MAX_CAP_REPRESENTABLE: () = assert!((1 << Self::SIZE_BITS) - 1 < Self::MAX_CAPACITY);
    const _FITS_IN_U32: () = assert!(Self::MAX_CAPACITY * 2 + Self::SIZE_BITS <= 64);

    const SIZE_MASK: u64 = !((!0) << Self::SIZE_BITS);

    pub const fn new() -> Self {
        PathBitString(0)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        (self.0 & Self::SIZE_MASK) as usize
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub const fn append(&self, dir: Direction) -> Option<Self> {
        if self.len() >= Self::MAX_CAPACITY {
            None
        } else {
            let mut written = self.write_index(self.len(), dir);
            written.0 += 1;
            Some(written)
        }
    }

    #[must_use]
    pub const fn pop(&self) -> Option<(Self, Direction)> {
        if self.is_empty() {
            None
        } else {
            let last = self.read_index(self.len() - 1);
            Some((Self(self.0 - 1), last))
        }
    }

    pub const fn inverse(&self) -> Self {
        let size_bits = self.0 & Self::SIZE_MASK;
        let len = self.len();

        // (64 - 2 * len - 6) zeros, (2 * len) ones, then 6 zeros.
        let contents_mask = (!((!0) << (2 * len))) << Self::SIZE_BITS;
        let mut content_bits = self.0 & contents_mask;

        // 6 zeros, (2 * len) bits to reinterpret, then (64 - 2 * len - 6) zeros.
        content_bits = content_bits.reverse_bits();
        // Back to leading zeros, bits, then 4 zeros.
        content_bits >>= 64 - 12 - 2 * len;

        let pair_firsts = 0x5555_5555_5555_5555 << Self::SIZE_BITS;
        let firsts = content_bits & pair_firsts;
        let seconds = (content_bits >> 1) & pair_firsts;

        let pair_xnor = firsts ^ seconds ^ pair_firsts;
        let up_down_mask = pair_xnor | (pair_xnor << 1);

        // To flip Up and Down, simply xor.
        content_bits ^= up_down_mask;

        Self(content_bits | size_bits)
    }

    pub const fn write_index(&self, index: usize, dir: Direction) -> Self {
        let bit_offset = 2 * index + Self::SIZE_BITS;
        let mut bits = self.0;
        bits &= !(0b11 << bit_offset);
        bits |= Self::dir_to_pattern(dir) << bit_offset;
        Self(bits)
    }

    pub const fn read_index(&self, index: usize) -> Direction {
        let bit_offset = 2 * index + Self::SIZE_BITS;
        let pattern = (self.0 >> bit_offset) & 0b11;
        Self::pattern_to_dir(pattern)
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

    pub fn iter(&self) -> impl Iterator<Item = Direction> {
        self.into_iter()
    }
}

impl Default for PathBitString {
    fn default() -> Self {
        Self::new()
    }
}

impl PathLike for PathBitString {
    fn append(&self, dir: Direction) -> Option<Self> {
        self.append(dir)
    }

    fn pop(&self) -> Option<(Self, Direction)> {
        self.pop()
    }

    fn inverse(&self) -> Self {
        self.inverse()
    }
}

impl PartialEq for PathBitString {
    fn eq(&self, other: &Self) -> bool {
        // We don't need to check if self.len() == other.len()
        // The mask will not or 1's over the len bits.
        let mask_out = (!0) << (2 * self.len() + 4);
        let mask = !mask_out;
        (self.0 & mask) == (other.0 & mask)
    }
}

impl Eq for PathBitString {}

impl<'a> IntoIterator for &'a PathBitString {
    type Item = Direction;
    type IntoIter = PathBitStringIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PathBitStringIter { string: self, i: 0 }
    }
}

pub struct PathBitStringIter<'a> {
    string: &'a PathBitString,
    i: usize,
}

impl Iterator for PathBitStringIter<'_> {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.string.len() {
            return None;
        }

        let dir = self.string.read_index(self.i);

        self.i += 1;
        Some(dir)
    }
}
