use super::WalkIsEmpty;
use crate::direction::Direction;
use crate::walk::enumeration::WalkEnum;
use crate::walk::traits::IsAWalk;

pub type ReducedWalk = Reduced<WalkEnum>;

// "Reduced" like a "word" from group theory.
pub struct Reduced<Walk: IsAWalk>(Walk);

impl<Walk: IsAWalk> IsAWalk for Reduced<Walk> {
    type PushError = Walk::PushError;

    fn new_empty() -> Self {
        Reduced(Walk::new_empty())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn peek(&self) -> Result<Direction, WalkIsEmpty> {
        self.0.peek()
    }

    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError> {
        if self
            .0
            .peek()
            .is_ok_and(|peeked| peeked.const_eq(dir.inverse()))
        {
            let _ = self.0.pop_mut();
            Ok(())
        } else {
            self.0.push_mut(dir)
        }
    }

    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty> {
        self.0.pop_mut()
    }
}
