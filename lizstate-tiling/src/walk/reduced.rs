use super::WalkIsEmpty;
use crate::direction::Direction;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkPartial;
use crate::walk::traits::IsAWalkRaw;

// "Reduced" like a "word" from group theory.
#[must_use]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reduced<Walk: IsAWalkRaw>(Walk);

impl<Walk: IsAWalkRaw> IsAWalkPartial for Reduced<Walk> {
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

impl<Walk> IntoIterator for Reduced<Walk>
where
    Walk: IsAWalkRaw,
    Walk: IntoIterator<Item = Direction>,
{
    type Item = Direction;
    type IntoIter = Walk::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Walk: IsAWalkRaw> IsAWalk for Reduced<Walk> {}
