use super::algorithms::Segment;
use super::AbsolutePosition;
use super::OctantRelative;
use super::RelativePosition;

#[derive(Debug)]
struct TrieNode {
    tile: OctantRelative,
    generator: OctantRelative,

    // up: FOVTrieEdge,
    diag: Option<usize>,
    run: Option<usize>,
}

impl TrieNode {
    fn new(tile: OctantRelative, generator: OctantRelative) -> Self {
        Self {
            tile,
            generator,
            diag: None,
            run: None,
        }
    }
}

// TODO: Lenient FOV.
// TODO: Make a lazy-static? I also doubt you will need to increase radius much more than it is already.
#[derive(Debug)]
pub struct StrictFOV {
    radius: u32,
    nodes: Vec<TrieNode>,
}

impl StrictFOV {
    pub fn new(radius: u32) -> Self {
        let mut partial = StrictFOV {
            radius: 0,
            nodes: vec![TrieNode::new(
                OctantRelative::ignore_octant(0, 0),
                OctantRelative::ignore_octant(0, 0),
            )],
        };

        for _ in 1..=radius {
            partial.increase_radius()
        }

        partial
    }

    fn increase_radius(&mut self) {
        self.radius += 1;
        let run = self.radius;
        for rise in 0..=run {
            let generator = OctantRelative::ignore_octant(rise, run);

            let (a, b) = (Segment { target: generator }).calculate();
            // TODO: Don't skip, assert node matches next, then peek the next?
            self.extend(0, generator, a.into_iter().skip(1));
            if let Some(b) = b {
                self.extend(0, generator, b.into_iter().skip(1));
            }
        }
    }

    fn extend<I: Iterator<Item = OctantRelative>>(
        &mut self,
        index: usize,
        generator: OctantRelative,
        mut iter: I,
    ) {
        // indexing should never panic. 0 is valid from construction, and any index contained in a node must be a valid node.
        let next = iter.next();
        if let Some(tile) = next {
            if tile.run == self.nodes[index].tile.run + 1
                && tile.rise == self.nodes[index].tile.rise
            {
                let subtree = match self.nodes[index].run {
                    Some(subtree) => subtree,
                    None => {
                        self.nodes.push(TrieNode::new(tile, generator));
                        self.nodes[index].run = Some(self.nodes.len() - 1);
                        self.nodes.len() - 1
                    }
                };
                self.extend(subtree, generator, iter);
            } else if tile.run == self.nodes[index].tile.run + 1
                && tile.rise == self.nodes[index].tile.rise + 1
            {
                let subtree = match self.nodes[index].diag {
                    Some(subtree) => subtree,
                    None => {
                        self.nodes.push(TrieNode::new(tile, generator));
                        self.nodes[index].diag = Some(self.nodes.len() - 1);
                        self.nodes.len() - 1
                    }
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
    pub fn get_field_of_view_tiles<F: Fn(AbsolutePosition) -> bool>(
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
            let mut frontier: Vec<usize> = vec![0];
            while let Some(current) = frontier.pop() {
                if self.nodes[current].generator.run > radius {
                    // In this subtree, we know that every tile we would output
                    // would be outside the radius. Rather than checking radius before outputting,
                    // we can just stop early.
                    continue;
                }

                let mut tile = self.nodes[current].tile;
                tile.octant = octant;
                let relative = RelativePosition::from(tile);

                // This is what makes StrictFov "strict".
                // This is equivalent to drawing a segment to the tile and checking everything in between.
                // Without this, you are allowed to draw a segment *through* the tile and see it.
                if self.nodes[current].generator == self.nodes[current].tile {
                    out.push(relative);
                }

                if !blocks_vision_relative(relative) {
                    if let Some(diag) = self.nodes[current].diag {
                        frontier.push(diag)
                    }
                    if let Some(run) = self.nodes[current].run {
                        frontier.push(run)
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
#[test]
fn new_strict_fov() {
    StrictFOV::new(5);
}

#[cfg(test)]
#[test]
#[should_panic]
fn get_fov_radius_too_large() {
    let fov = StrictFOV::new(0);
    fov.get_field_of_view_tiles(AbsolutePosition::new(0, 0), 1, |_| true);
}

#[cfg(test)]
#[test]
fn get_fov() {
    let fov = StrictFOV::new(2);
    let blocks_vision = |_| false;
    let tiles = fov.get_field_of_view_tiles(AbsolutePosition::new(0, 0), 2, blocks_vision);

    for x in -2..=2 {
        for y in -2..=2 {
            assert!(
                tiles.contains(&AbsolutePosition::new(x, y)),
                "{} {} {:?}",
                x,
                y,
                tiles
            )
        }
    }

    let mut tiles = tiles;
    tiles.sort_by_key(|pos| (pos.x, pos.y));
    tiles.dedup();
    assert_eq!(tiles.len(), 5 * 5)
    // Because we checked 25 unique items, and the vec has length 25, we have set equality.
}

#[cfg(test)]
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

#[cfg(test)]
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
