use super::PathAlreadyEmpty;
use crate::direction::Direction;
use crate::path::BoundedPathLike;

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Efficient<Path: BoundedPathLike>(Path)
where
    Path: BoundedPathLike;

impl<Path> Efficient<Path>
where
    Path: BoundedPathLike,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wrap(path: &Path) -> Self {
        Self(path.cancel_inverses())
    }
}

impl<Path> IntoIterator for Efficient<Path>
where
    Path: BoundedPathLike,
{
    type Item = Path::Item;
    type IntoIter = Path::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Path> BoundedPathLike for Efficient<Path>
where
    Path: BoundedPathLike,
{
    const MAX_CAPACITY: usize = Path::MAX_CAPACITY;

    fn push(&self, dir: Direction) -> Option<Self> {
        self.0.push_or_cancel(dir).map(Efficient)
    }

    fn pop(&self) -> Result<(Self, Direction), PathAlreadyEmpty> {
        self.0.pop().map(|(a, b)| (Efficient(a), b))
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
