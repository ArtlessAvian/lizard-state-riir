use std::vec::Vec;

use crate::direction::Direction;
use crate::walk::WalkIsEmpty;
use crate::walk::traits::IsAWalk;

pub enum Never {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub struct WalkVec {
    vec: Vec<Direction>,
}

impl IsAWalk for WalkVec {
    type PushError = Never;

    fn new_empty() -> Self {
        Self { vec: Vec::new() }
    }

    fn len(&self) -> usize {
        self.vec.len()
    }

    fn peek(&self) -> Result<Direction, WalkIsEmpty> {
        self.vec.last().copied().ok_or(WalkIsEmpty)
    }

    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError> {
        self.vec.push(dir);
        Ok(())
    }

    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty> {
        self.vec.pop().ok_or(WalkIsEmpty)
    }
}
