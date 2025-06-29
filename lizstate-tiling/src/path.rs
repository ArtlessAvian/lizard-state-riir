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
    fn push(&self, dir: Direction) -> Option<Self>;

    fn pop(&self) -> Option<(Self, Direction)>;

    #[must_use]
    fn inverse(&self) -> Self {
        let mut out = Self::default();
        let mut copy = *self;
        while let Some((next, dir)) = copy.pop() {
            copy = next;
            out = out
                .push(dir.inverse())
                .expect("we cannot pop more than MAX_CAPACITY elements before the loop terminates");
        }
        out
    }

    fn extend(&self, path: impl IntoIterator<Item = Direction>) -> Option<Self> {
        path.into_iter().try_fold(*self, |out, dir| out.push(dir))
    }
}
