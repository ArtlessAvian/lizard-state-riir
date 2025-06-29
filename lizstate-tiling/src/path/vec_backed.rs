use crate::direction::Direction;
use crate::path::PathAlreadyEmpty;

/// A mutable Vec for Directions.
pub trait UnboundedPathLike
where
    Self: Default
        + Clone
        + Eq
        + IntoIterator<Item = Direction>
        + FromIterator<Direction>
        + Extend<Direction>,
{
    fn push(&mut self, dir: Direction);

    /// # Errors
    /// Returns Err when already empty.
    fn pop(&mut self) -> Result<Direction, PathAlreadyEmpty>;

    fn iter(&self) -> impl DoubleEndedIterator<Item = Direction>;

    /// Returns the path backwards with inverse directions.
    #[must_use]
    fn inverse(&self) -> Self {
        self.iter().rev().map(Direction::inverse).collect()
    }

    /// Cancels if the last direction is opposite, otherwise pushes.
    fn push_or_cancel(&mut self, dir: Direction) {
        if let Ok(last) = self.pop() {
            if last.inverse() == dir {
                // we already popped!
            } else {
                // unpop, then push
                self.push(last);
                self.push(dir);
            }
        } else {
            self.push(dir);
        }
    }

    /// Cleans up redundant steps in the path.
    fn cancel_inverses(&mut self) {
        let mut to_replace = Self::default();
        for dir in self.iter() {
            to_replace.push_or_cancel(dir);
        }
        let _ = std::mem::replace(self, to_replace);
    }
}
