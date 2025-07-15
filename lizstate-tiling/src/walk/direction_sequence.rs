#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "type can grow maybe (and still be copy)"
)]

use lizstate_sequence_enumeration::SequenceEmpty;
use lizstate_sequence_enumeration::SequenceFull;
use lizstate_sequence_enumeration::SequenceOf;

use crate::direction::Direction;
use crate::walk::WalkIsEmpty;
use crate::walk::WalkIsFull;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkPartial;
use crate::walk::traits::IsAWalkRaw;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct WalkEnum(SequenceOf<Direction>);

impl IsAWalkPartial for WalkEnum {
    type PushError = WalkIsFull;

    fn new_empty() -> Self
    where
        Self: Sized,
    {
        Self(SequenceOf::new_empty())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn peek(&self) -> Result<Direction, WalkIsEmpty> {
        self.0.peek().map_err(|SequenceEmpty| WalkIsEmpty)
    }

    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError> {
        self.0.push(dir).map_err(|SequenceFull| WalkIsFull)
    }

    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty> {
        self.0.pop().map_err(|SequenceEmpty| WalkIsEmpty)
    }

    fn prefix_mut(&mut self) {
        _ = self.0.pop();
    }
}

impl IntoIterator for WalkEnum {
    type Item = Direction;
    type IntoIter = WalkEnumIterator;

    fn into_iter(self) -> Self::IntoIter {
        // HACK: We want a queue, but we have a stack interface.
        // Inverting both reverses the sequence and inverts each direction.
        // We'll need to revert the directions later.
        let mut inverse = self;
        inverse.inverse_mut();
        WalkEnumIterator { inverse }
    }
}

pub struct WalkEnumIterator {
    inverse: WalkEnum,
}

impl Iterator for WalkEnumIterator {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        // HACK: We can pop directions the inverse of our path. We can revert individual directions now.
        self.inverse.pop_mut().ok().map(Direction::inverse)
    }
}

impl IsAWalk for WalkEnum {}

impl IsAWalkRaw for WalkEnum {}
