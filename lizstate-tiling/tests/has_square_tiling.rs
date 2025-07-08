#![allow(dead_code, reason = "we just to compile these for testing and demos")]

use std::collections::HashSet;
use std::ops::Range;

use lizstate_tiling::direction::Direction;
use lizstate_tiling::tiling::HasSquareTiling;
use lizstate_tiling::tiling::IsATile;

#[derive(Clone, Copy, PartialEq, Eq)]
struct LineNode(i32);
impl IsATile for LineNode {}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct LineGraph {}

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

/// Homomorphic to the `LineGraph` of course.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct VerticalLineGraph {}

impl HasSquareTiling<LineNode> for VerticalLineGraph {
    fn get_origin(&self) -> LineNode {
        LineNode(0)
    }

    fn step(&self, tile: &LineNode, dir: Direction) -> Option<LineNode> {
        match dir {
            Direction::Up => Some(LineNode(tile.0 + 1)),
            Direction::Down => Some(LineNode(tile.0 - 1)),
            Direction::Right => None,
            Direction::Left => None,
        }
    }
}

#[derive(Default, PartialEq, Eq)]
/// Range doesn't impl Copy? Because they're iterators? Woag.
struct BoundedGridGraph(Range<i32>, Range<i32>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GridNode(i32, i32);
impl IsATile for GridNode {}

impl HasSquareTiling<GridNode> for &BoundedGridGraph {
    fn get_origin(&self) -> GridNode {
        assert!(self.0.contains(&0));
        assert!(self.1.contains(&0));
        GridNode(0, 0)
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
struct GraphIntersection<SpaceA, SpaceB>(SpaceA, SpaceB);

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

#[test]
fn grid_intersection() {
    let grid_a = BoundedGridGraph(0..2, -2..2);
    let grid_b = BoundedGridGraph(-2..2, 0..2);

    // This should be effectively BoundedGridGraph(0..2, 0..2)
    let intersection = GraphIntersection(&grid_a, &grid_b);
    let origin: IntersectionElement<GridNode, GridNode> = intersection.get_origin();

    assert!(intersection.step(&origin, Direction::Down).is_none());
    assert!(intersection.step(&origin, Direction::Left).is_none());
}

#[derive(Clone, PartialEq, Eq)]
struct OutdoorMap(HashSet<GridNode>);

impl HasSquareTiling<GridNode> for &OutdoorMap {
    fn get_origin(&self) -> GridNode {
        let out = GridNode(0, 0);
        assert!(!self.0.contains(&out));
        out
    }

    fn step(&self, tile: &GridNode, dir: Direction) -> Option<GridNode> {
        let tentative = match dir {
            Direction::Up => GridNode(tile.0, tile.1 + 1),
            Direction::Down => GridNode(tile.0, tile.1 - 1),
            Direction::Right => GridNode(tile.0 + 1, tile.1),
            Direction::Left => GridNode(tile.0 - 1, tile.1),
        };
        if !self.0.contains(&tentative) {
            Some(tentative)
        } else {
            None
        }
    }
}

#[test]
fn outdoor_map() {
    let mut set = HashSet::new();
    set.insert(GridNode(1, 1));

    let map = OutdoorMap(set);

    let origin = (&map).get_origin();
    let right = (&map).step(&origin, Direction::Right).unwrap();
    assert_eq!((&map).step(&right, Direction::Up), None);
}

// // References should probably not impl GridNode.
// // Causes lifetime issues.
// #[derive(Clone, PartialEq, Eq)]
// struct IndoorMap(Vec<GridNode>);

// impl IsATile for &GridNode {}

// impl HasSquareTiling<&GridNode> for &IndoorMap {
//     fn get_origin(&self) -> &GridNode {
//         for node in &self.0 {
//             if *node == GridNode(0, 0) {
//                 return node;
//             }
//         }
//         panic!();
//     }

//     fn step(&self, tile: &&GridNode, dir: Direction) -> Option<GridNode> {
//         let tentative = match dir {
//             Direction::Up => GridNode(tile.0, tile.1 + 1),
//             Direction::Down => GridNode(tile.0, tile.1 - 1),
//             Direction::Right => GridNode(tile.0 + 1, tile.1),
//             Direction::Left => GridNode(tile.0 - 1, tile.1),
//         };
//         if self.0.contains(&tentative) {
//             Some(tentative)
//         } else {
//             None
//         }
//     }
// }
