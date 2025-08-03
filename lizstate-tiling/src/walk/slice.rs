use crate::direction::Direction;
use crate::walk::WalkIsEmpty;
use crate::walk::WalkIsFull;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkMut;

const EMPTY: [Direction; 0] = [];

/// A slice with a pop function.
///
/// Realistically, you are only popping a slice for test code.
///
/// Slices already implement `IntoIterator` and all the traits we care about, so this is straightforward.
#[must_use]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SliceWalk<'a>(&'a [Direction]);

impl<'a> IntoIterator for SliceWalk<'a> {
    type Item = Direction;
    type IntoIter = core::iter::Copied<core::slice::Iter<'a, Direction>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl IsAWalk for SliceWalk<'_> {
    fn new_empty() -> Self
    where
        Self: Sized,
    {
        SliceWalk(&EMPTY)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn peek(&self) -> Result<Direction, WalkIsEmpty> {
        self.0.last().copied().ok_or(WalkIsEmpty)
    }
}

impl IsAWalkMut for SliceWalk<'_> {
    type PushError = WalkIsFull;

    fn push_mut(&mut self, _: Direction) -> Result<(), Self::PushError> {
        Err(WalkIsFull)
    }

    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty> {
        if let Ok(peeked) = self.peek() {
            let subslice = &self.0[0..self.len() - 1];
            self.0 = subslice;
            Ok(peeked)
        } else {
            Err(WalkIsEmpty)
        }
    }
}
