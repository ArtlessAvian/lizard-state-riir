use core::marker::PhantomData;

use crate::path::BoundedPathLike;
use crate::tiling::HasSquareTiling;
use crate::tiling::IsATile;

struct NaivePathEquality<Path, Space, Tile>
where
    Path: BoundedPathLike,
    Space: HasSquareTiling<Tile>,
    Tile: IsATile,
{
    path: Path,
    space: Space,
    tile: PhantomData<Tile>,
}

impl<Path, Space, Tile> PartialEq for NaivePathEquality<Path, Space, Tile>
where
    Path: BoundedPathLike,
    Space: HasSquareTiling<Tile>,
    Tile: IsATile,
{
    fn eq(&self, other: &Self) -> bool {
        self.space == other.space
            && self.space.skip_path(&self.space.get_origin(), self.path)
                == other.space.skip_path(&other.space.get_origin(), other.path)
    }
}
