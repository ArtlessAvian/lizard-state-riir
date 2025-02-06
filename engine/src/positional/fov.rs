use std::ops::Index;
use std::ops::IndexMut;

use tracing::instrument;

use super::algorithms::Segment;
use super::AbsolutePosition;
use super::InsideOctant;
use super::RelativeOctantified;
use super::RelativePosition;

#[derive(Debug, Clone, Copy)]
struct TrieNodeIndex(usize);

#[derive(Debug)]
struct TrieNode {
    tile: InsideOctant,
    generator: InsideOctant,

    // up: FOVTrieEdge,
    diag: Option<TrieNodeIndex>,
    run: Option<TrieNodeIndex>,
}

impl TrieNode {
    fn new(tile: InsideOctant, generator: InsideOctant) -> Self {
        Self {
            tile,
            generator,
            diag: None,
            run: None,
        }
    }
}

#[derive(Debug)]
struct NodeArena(Vec<TrieNode>);

impl NodeArena {
    // Append/emplace only.
    // Without removal, any TrieNodeIndex created by the arena will always be vali.
    fn emplace(&mut self, node: TrieNode) -> TrieNodeIndex {
        self.0.push(node);
        TrieNodeIndex(self.0.len() - 1)
    }
}

impl Index<TrieNodeIndex> for NodeArena {
    type Output = TrieNode;

    fn index(&self, index: TrieNodeIndex) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<TrieNodeIndex> for NodeArena {
    fn index_mut(&mut self, index: TrieNodeIndex) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

// TODO: Lenient FOV.
// TODO: Make a lazy-static? I also doubt you will need to increase radius much more than it is already.
#[derive(Debug)]
pub struct StrictFOV {
    radius: u32,
    nodes: NodeArena,
}

impl StrictFOV {
    #[must_use]
    pub(crate) fn new(radius: u32) -> Self {
        let mut partial = StrictFOV {
            radius: 0,
            nodes: NodeArena(vec![TrieNode::new(
                InsideOctant::new(0, 0),
                InsideOctant::new(0, 0),
            )]),
        };

        for _ in 1..=radius {
            partial.increase_radius();
        }

        partial
    }

    fn increase_radius(&mut self) {
        self.radius += 1;
        let run = self.radius;
        for rise in 0..=run {
            let generator = InsideOctant::new(rise, run);

            let (a, b) = Segment::calculate(generator);
            // TODO: Don't skip, assert node matches next, then peek the next?
            self.extend(TrieNodeIndex(0), generator, a.into_iter().skip(1));
            if let Some(b) = b {
                self.extend(TrieNodeIndex(0), generator, b.into_iter().skip(1));
            }
        }
    }

    fn extend<I: Iterator<Item = InsideOctant>>(
        &mut self,
        index: TrieNodeIndex,
        generator: InsideOctant,
        mut iter: I,
    ) {
        // indexing should never panic. 0 is valid from construction, and any index contained in a node must be a valid node.
        let next = iter.next();
        if let Some(tile) = next {
            if tile.run == self.nodes[index].tile.run + 1
                && tile.rise == self.nodes[index].tile.rise
            {
                // let subtree = self.nodes[index]
                //     .run
                //     .get_or_insert_with(|| self.nodes.emplace(TrieNode::new(tile, generator)));
                let subtree = if let Some(subtree) = self.nodes[index].run {
                    subtree
                } else {
                    let subtree = self.nodes.emplace(TrieNode::new(tile, generator));
                    self.nodes[index].run = Some(subtree);
                    subtree
                };
                self.extend(subtree, generator, iter);
            } else if tile.run == self.nodes[index].tile.run + 1
                && tile.rise == self.nodes[index].tile.rise + 1
            {
                let subtree = if let Some(subtree) = self.nodes[index].diag {
                    subtree
                } else {
                    let subtree = self.nodes.emplace(TrieNode::new(tile, generator));
                    self.nodes[index].diag = Some(subtree);
                    subtree
                };
                self.extend(subtree, generator, iter);
            } else if tile.run == self.nodes[index].tile.run
                && tile.rise == self.nodes[index].tile.rise + 1
            {
                unimplemented!("Segments aren't expected to step upwards.");
            } else {
                panic!("Segment is discontinuous!");
            }
        }
    }

    /// Returns a Vec of all tiles visible. A tile that blocks vision is itself visible, but there's no vision past it.
    /// There is no attempt to dedup tiles!
    pub(crate) fn get_field_of_view_tiles<F: Fn(AbsolutePosition) -> bool>(
        &self,
        center: AbsolutePosition,
        radius: u32,
        blocks_vision: F,
    ) -> Vec<AbsolutePosition> {
        self.get_field_of_view_tiles_relative(radius, |x| blocks_vision(center + x))
            .into_iter()
            .map(|x| center + x)
            .collect()
    }

    #[instrument(skip_all)]
    fn get_field_of_view_tiles_relative<F: Fn(RelativePosition) -> bool>(
        &self,
        radius: u32,
        blocks_vision_relative: F,
    ) -> Vec<RelativePosition> {
        assert!(
            radius <= self.radius,
            "caller requested radius {}, we have precalculated radius {}",
            radius,
            self.radius
        );

        let mut out = Vec::new();
        for octant in 0..8 {
            // DFS through the trie. BFS would return tiles in order of radius, which is nice.
            let mut frontier = vec![TrieNodeIndex(0)];
            while let Some(current) = frontier.pop() {
                if self.nodes[current].generator.run > radius {
                    // In this subtree, we know that every tile we would output
                    // would be outside the radius. Rather than checking radius before outputting,
                    // we can just stop early.
                    continue;
                }

                let relative = RelativePosition::from(RelativeOctantified {
                    inside: self.nodes[current].tile,
                    octant,
                });

                // This is what makes StrictFov "strict".
                // This is equivalent to drawing a segment to the tile and checking everything in between.
                // Without this, you are allowed to draw a segment *through* the tile and see it.
                if self.nodes[current].generator == self.nodes[current].tile {
                    out.push(relative);
                }

                if !blocks_vision_relative(relative) {
                    if let Some(diag) = self.nodes[current].diag {
                        frontier.push(diag);
                    }
                    if let Some(run) = self.nodes[current].run {
                        frontier.push(run);
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod test {
    use super::StrictFOV;
    use crate::positional::AbsolutePosition;

    #[test]
    fn new_strict_fov() {
        _ = StrictFOV::new(5);
    }

    #[test]
    #[should_panic(expected = "caller requested radius 1, we have precalculated radius 0")]
    fn get_fov_radius_too_large() {
        let fov = StrictFOV::new(0);
        fov.get_field_of_view_tiles(AbsolutePosition::new(0, 0), 1, |_| true);
    }

    #[test]
    fn get_fov() {
        let fov = StrictFOV::new(2);
        let blocks_vision = |_| false;
        let tiles = fov.get_field_of_view_tiles(AbsolutePosition::new(0, 0), 2, blocks_vision);

        for x in -2..=2 {
            for y in -2..=2 {
                assert!(
                    tiles.contains(&AbsolutePosition::new(x, y)),
                    "{x} {y} {tiles:?}"
                );
            }
        }

        let mut tiles = tiles;
        tiles.sort_by_key(|pos| (pos.x, pos.y));
        tiles.dedup();
        assert_eq!(tiles.len(), 5 * 5);
        // Because we checked 25 unique items, and the vec has length 25, we have set equality.
    }

    #[test]
    fn get_fov_with_obstruction() {
        let fov = StrictFOV::new(2);
        let blocks_vision = |pos: AbsolutePosition| pos.x > 0 && -1 <= pos.y && pos.y <= 1;
        let tiles = fov.get_field_of_view_tiles(AbsolutePosition::new(0, 0), 2, blocks_vision);

        // @#
        // .#
        // ..

        for y in -2..=2 {
            assert!(tiles.contains(&AbsolutePosition::new(0, y))); // open space
            assert!(tiles.contains(&AbsolutePosition::new(1, y))); // visible wall. (or nothing.)
            assert!(!tiles.contains(&AbsolutePosition::new(2, y))); // space blocked by wall.
        }
    }

    #[test]
    fn get_fov_with_direct_only() {
        let fov = StrictFOV::new(4);
        let blocks_vision = |pos: AbsolutePosition| pos.x / 3 != pos.y;
        let tiles = fov.get_field_of_view_tiles(AbsolutePosition::new(0, 0), 4, blocks_vision);

        // We can see (4, 1) passing through (3, 1), but we can't see (3, 1) itself.
        // This causes a discontinuity, which is ugly but whatever. This makes sense for "game logic" like hitting each other.
        // @..##
        // ### .

        assert!(tiles.contains(&AbsolutePosition::new(4, 1)));
        assert!(!tiles.contains(&AbsolutePosition::new(3, 1)));
    }
}
