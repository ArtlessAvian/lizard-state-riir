use crate::direction::Direction;
use crate::walk::WalkIsEmpty;

#[must_use]
pub trait IsAWalk {
    type PushError;

    fn new_empty() -> Self;

    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    fn len(&self) -> usize;

    /// The direction at the end of the walk.
    /// # Errors
    /// Nothing to peek at.
    fn peek(&self) -> Result<Direction, WalkIsEmpty>;

    /// # Errors
    /// Implementor cannot represent the extended walk.
    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError>;

    /// # Errors
    /// Nothing to pop from walk.
    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty>;
}

pub trait IsAWalkCopy: IsAWalk + Copy {
    /// # Errors
    /// Implementor cannot represent the extended walk.
    fn push_copy(&self, dir: Direction) -> Result<Self, Self::PushError>;

    /// # Errors
    /// Nothing to pop from walk.
    fn pop_copy(&self) -> Result<(Self, Direction), WalkIsEmpty>;
}
