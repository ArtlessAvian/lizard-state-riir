use core::marker::PhantomData;

use crate::path::PathLike;
use crate::tiling::HasSquareTiling;
use crate::tiling::Tile;

struct NaivePathEquality<Path, Space, T>
where
    Path: PathLike,
    Space: HasSquareTiling<T>,
    T: Tile,
{
    path: Path,
    space: Space,
    t: PhantomData<T>,
}

impl<Path, Space, T> PartialEq for NaivePathEquality<Path, Space, T>
where
    Path: PathLike,
    Space: HasSquareTiling<T>,
    T: Tile,
{
    fn eq(&self, other: &Self) -> bool {
        self.space == other.space
            && self.space.skip_path(&self.space.get_origin(), self.path)
                == other.space.skip_path(&other.space.get_origin(), other.path)
    }
}
