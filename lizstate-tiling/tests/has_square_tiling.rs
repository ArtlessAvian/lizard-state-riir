#![allow(dead_code)]

use std::ops::Range;

use lizstate_tiling::direction::Direction;
use lizstate_tiling::tiling::HasSquareTiling;
use lizstate_tiling::tiling::IsATile;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct LineGraph {}

#[derive(Clone, Copy, PartialEq, Eq)]
struct LineNode(i32);
impl IsATile for LineNode {}

impl HasSquareTiling<LineNode> for LineGraph {
    fn get_origin(&self) -> LineNode {
        LineNode(0)
    }

    fn step(&self, tile: &LineNode, dir: Direction) -> Option<LineNode> {
        match dir {
            Direction::Up => None,
            Direction::Down => None,
            Direction::Right => Some(LineNode(tile.0 + 1)),
            Direction::Left => Some(LineNode(tile.0 - 1)),
        }
    }
}

#[derive(Default, PartialEq, Eq)]
/// Range doesn't impl Copy? Because they're iterators? Woag.
struct BoundedGridGraph(Range<i32>, Range<i32>);

#[derive(Clone, Copy, PartialEq, Eq)]
struct GridNode(i32, i32);
impl IsATile for GridNode {}

impl HasSquareTiling<GridNode> for &BoundedGridGraph {
    fn get_origin(&self) -> GridNode {
        GridNode(
            {
                let Range { start, end } = self.0;
                (start + end) / 2
            },
            {
                let Range { start, end } = self.1;
                (start + end) / 2
            },
        )
    }

    fn step(&self, tile: &GridNode, dir: Direction) -> Option<GridNode> {
        let temp = match dir {
            Direction::Up => GridNode(tile.0, tile.1 + 1),
            Direction::Down => GridNode(tile.0, tile.1 - 1),
            Direction::Right => GridNode(tile.0 + 1, tile.1),
            Direction::Left => GridNode(tile.0 - 1, tile.1),
        };

        if self.0.contains(&temp.0) && self.1.contains(&temp.1) {
            Some(temp)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct GraphIntersection<A, B>(A, B);

#[derive(Clone, Copy, PartialEq, Eq)]
struct IntersectionElement<TileA: IsATile, TileB: IsATile>(TileA, TileB);
impl<TileA: IsATile, TileB: IsATile> IsATile for IntersectionElement<TileA, TileB> {}

impl<SpaceA, SpaceB, TileA, TileB> HasSquareTiling<IntersectionElement<TileA, TileB>>
    for GraphIntersection<SpaceA, SpaceB>
where
    SpaceA: HasSquareTiling<TileA>,
    SpaceB: HasSquareTiling<TileB>,
    TileA: IsATile,
    TileB: IsATile,
{
    fn get_origin(&self) -> IntersectionElement<TileA, TileB> {
        IntersectionElement(self.0.get_origin(), self.1.get_origin())
    }

    fn step(
        &self,
        tile: &IntersectionElement<TileA, TileB>,
        dir: Direction,
    ) -> Option<IntersectionElement<TileA, TileB>> {
        Some(IntersectionElement(
            self.0.step(&tile.0, dir)?,
            self.1.step(&tile.1, dir)?,
        ))
    }
}
