use core::hash::Hash;

use crate::direction::Direction;
use crate::walk::WalkIsEmpty;

/// Mutable interface for a
#[must_use]
pub trait IsAWalkPartial {
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

    // *************** Mutable Interface ***************

    /// # Errors
    /// Implementor cannot represent the extended walk.
    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError>;

    /// # Errors
    /// Nothing to pop from walk.
    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty>;

    // *************** Immutable Interface ***************
    // Default impls are fine.
    // If wanted, you can override the immutable functions.
    // And have the mutable functions call the copy ones.

    /// # Errors
    /// Implementor cannot represent the extended walk.
    fn push_copy(&self, dir: Direction) -> Result<Self, Self::PushError>
    where
        Self: Copy,
    {
        let mut copy = *self;
        let result = copy.push_mut(dir);
        result.map(|()| copy)
    }

    /// # Errors
    /// Nothing to pop from walk.
    fn pop_copy(&self) -> Result<(Self, Direction), WalkIsEmpty>
    where
        Self: Copy,
    {
        let mut copy = *self;
        let result = copy.pop_mut();
        result.map(|dir| (copy, dir))
    }
}

/// Very narrow subtrait. All of these should be easily implementable.
#[must_use]
pub trait IsAWalk
where
    Self: IsAWalkPartial,
    Self: Clone + PartialEq + Eq + Hash,
    Self: IntoIterator<Item = Direction>,
{
}
