use crate::path::Direction;
use crate::path::PathLike;

/// A string of `Directions`.
///
/// A bit space wasteful, but its alright.
/// Mostly exists for testing reference.
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

impl PathLike for PathString {
    fn append(&self, dir: Direction) -> Option<Self> {
        self.append(dir)
    }

    fn pop(&self) -> Option<(Self, Direction)> {
        self.pop()
    }

    fn inverse(&self) -> Self {
        self.inverse()
    }

    fn iter(&self) -> impl Iterator<Item = Direction> {
        (self).into_iter()
    }
}
