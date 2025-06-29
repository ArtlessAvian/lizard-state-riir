#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "type can grow maybe (and still be copy)"
)]

use crate::direction::Direction;

pub mod array_backed;
pub mod bits_backed;

pub trait PathLike
where
    Self: Default + Copy + Eq + IntoIterator<Item = Direction>,
{
    fn append(&self, dir: Direction) -> Option<Self>;

    fn pop(&self) -> Option<(Self, Direction)>;

    #[must_use]
    fn inverse(&self) -> Self;
}
