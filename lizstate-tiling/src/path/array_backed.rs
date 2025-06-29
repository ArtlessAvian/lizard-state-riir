use crate::path::Direction;
use crate::path::PathLike;

/// A string of `Directions`.
///
/// A bit space wasteful, but its alright.
/// Mostly exists for testing reference.
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct PathString<const N: usize>(usize, [Direction; N]);

// Const implementations. Bleh.
impl<const N: usize> PathString<N> {
    pub const fn new_empty() -> Self {
        PathString(0, [Direction::Up; N])
    }

    pub const fn new_from_array(array: [Direction; N]) -> Self {
        Self(N, array)
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
        self.len() >= N
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
    pub const fn push(&self, dir: Direction) -> Option<Self> {
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
        let mut array = [Direction::Up; N];

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

impl<const N: usize> Default for PathString<N> {
    fn default() -> Self {
        Self::new_empty()
    }
}

// Since PathString::pop was lazy, we can't derive Eq.
impl<const N: usize> PartialEq for PathString<N> {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0) && (self.as_slice() == other.as_slice())
    }
}

impl<const N: usize> Eq for PathString<N> {}

impl<const N: usize> IntoIterator for PathString<N> {
    type Item = Direction;
    type IntoIter = core::iter::Take<core::array::IntoIter<Direction, N>>;

    fn into_iter(self) -> Self::IntoIter {
        self.1.into_iter().take(self.0)
    }
}

impl<const N: usize> PathLike for PathString<N> {
    const MAX_CAPACITY: usize = N;

    fn push(&self, dir: Direction) -> Option<Self> {
        self.push(dir)
    }

    fn pop(&self) -> Option<(Self, Direction)> {
        self.pop()
    }
}
