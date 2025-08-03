use core::error::Error;
use core::hash::Hash;

use crate::direction::Direction;
use crate::walk::WalkIsEmpty;

#[must_use]
pub trait IsAWalkMut
where
    Self: Clone + PartialEq + Eq + Hash,
    Self: IntoIterator<Item = Direction>,
{
    type PushError: Error;

    fn new_empty() -> Self
    where
        Self: Sized;

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

    /// Removes one element from the list, or does nothing.
    fn prefix_mut(&mut self) {
        _ = self.pop_mut();
    }

    /// Removes half the elements and puts them in a new Walk.
    /// `self` contains the first half.
    #[must_use]
    fn split_mut(&mut self) -> Self
    where
        Self: Sized,
    {
        let out_len = self.len() / 2;
        let mut out = Self::new_empty();
        for _ in 0..out_len {
            let popped = self.pop_mut().expect("we can pop out_len elements");
            out.push_mut(popped).expect("we can push out_len elements");
        }
        out.inverse_mut();
        out
    }

    /// Replaces self with the walk in the opposite direction.
    /// # Panics
    /// Self should be able to store all walks of length `self.len()`
    fn inverse_mut(&mut self)
    where
        Self: Sized,
    {
        let mut out = Self::new_empty();
        while let Ok(popped) = self.pop_mut() {
            out.push_mut(popped.inverse())
                .expect("implementor should be able to store all paths of length `self.len()`");
        }
        *self = out;
    }

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

    /// Returns a new list without the top element, or an empty list.
    #[must_use]
    fn prefix_copy(&self) -> Self
    where
        Self: Copy,
    {
        let mut copy = *self;
        copy.prefix_mut();
        copy
    }

    #[must_use]
    fn split_copy(&self) -> (Self, Self)
    where
        Self: Copy,
    {
        let mut copy = *self;
        let second = copy.split_mut();
        (copy, second)
    }

    /// Replaces self with the walk in the opposite direction.
    /// # Panics
    /// Self should be able to store all walks of length `self.len()`
    #[must_use]
    fn inverse_copy(&self) -> Self
    where
        Self: Copy,
    {
        let mut copy = *self;
        copy.inverse_mut();
        copy
    }

    /// Creates a new Self from an iterator.
    /// # Errors
    /// Self cannot represent path with combined length.
    /// This will also drop both self and iter.
    fn try_new_from_iter(
        iter: impl IntoIterator<Item = Direction>,
    ) -> Result<Self, Self::PushError> {
        let mut out = Self::new_empty();
        for dir in iter {
            out.push_mut(dir)?;
        }
        Ok(out)
    }

    /// Takes ownership of `self` and some Iterator of Directions.
    /// If successful, returns a Self with the two contents.
    /// # Errors
    /// Self cannot represent path with combined length.
    /// This will also drop both self and iter, which are in invalid states.
    fn try_extend(
        self,
        iter: impl IntoIterator<Item = Direction>,
    ) -> Result<Self, Self::PushError> {
        let mut out = self;
        for dir in iter {
            out.push_mut(dir)?;
        }
        Ok(out)
    }

    /// Takes ownership of `self` and another of the same type.
    /// If successful, returns a Self with the two contents.
    /// # Errors
    /// Self cannot represent path with combined length.
    /// This will also drop both self and other, which are in invalid states.
    fn try_append(self, other: Self) -> Result<Self, Self::PushError> {
        let mut out = self;
        for dir in other {
            out.push_mut(dir)?;
        }
        Ok(out)
    }
}

/// Marker trait for *NOT WRAPPERS*
///
/// Similar to nalgebra's storage stuff.
pub trait IsAWalkRaw: IsAWalkMut {}
