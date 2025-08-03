#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "type can grow maybe (and still be copy)"
)]

use lizstate_sequence_enumeration::SequenceEmpty;
use lizstate_sequence_enumeration::SequenceFull;
use lizstate_sequence_enumeration::element_deque::PackedDeque;

use crate::direction::Direction;
use crate::walk::WalkIsEmpty;
use crate::walk::WalkIsFull;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkMut;
use crate::walk::traits::IsAWalkRaw;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct WalkEnum(PackedDeque<Direction, 4, 31>);

impl IsAWalk for WalkEnum {
    fn new_empty() -> Self
    where
        Self: Sized,
    {
        Self(PackedDeque::new_empty())
    }

    fn len(&self) -> usize {
        self.0.len() as usize
    }

    fn peek(&self) -> Result<Direction, WalkIsEmpty> {
        self.0.peek_low().map_err(|SequenceEmpty| WalkIsEmpty)
    }
}

impl IsAWalkMut for WalkEnum {
    type PushError = WalkIsFull;

    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError> {
        self.0.push_low(dir).map_err(|SequenceFull| WalkIsFull)
    }

    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty> {
        // HACK: omg
        let peek = self.0.peek_low().ok();
        self.0
            .pop_low()
            .map_err(|SequenceEmpty| WalkIsEmpty)
            .map(|()| peek.unwrap())
    }
}

impl IntoIterator for WalkEnum {
    type Item = Direction;
    type IntoIter = WalkEnumIter;

    fn into_iter(self) -> Self::IntoIter {
        WalkEnumIter(self)
    }
}

pub struct WalkEnumIter(WalkEnum);

impl Iterator for WalkEnumIter {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        // HACK: why did i think seperating peek and pop was a good idea.
        let peek = self.0.0.peek_high().ok()?;
        _ = self.0.0.pop_high();
        Some(peek)
    }
}

impl IsAWalkRaw for WalkEnum {}
