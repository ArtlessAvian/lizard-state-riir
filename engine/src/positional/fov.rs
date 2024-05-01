use super::{algorithms::Segment, OctantRelative};

type TrieEdge = Option<Box<TrieNode>>;

#[derive(Debug)]
struct TrieNode {
    tile: OctantRelative,
    generator: OctantRelative,

    // up: FOVTrieEdge,
    diag: TrieEdge,
    run: TrieEdge,
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

    fn extend<I: Iterator<Item = OctantRelative>>(
        &mut self,
        generator: OctantRelative,
        mut iter: I,
    ) {
        let next = iter.next();
        if let Some(tile) = next {
            if tile.run == self.tile.run + 1 && tile.rise == self.tile.rise {
                let subtree = match &mut self.run {
                    Some(subtree) => subtree,
                    None => {
                        self.run = Some(Box::new(TrieNode::new(tile, generator)));
                        self.run.as_mut().unwrap()
                    }
                };
                subtree.extend(generator, iter);
            } else if tile.run == self.tile.run + 1 && tile.rise == self.tile.rise + 1 {
                let subtree = match &mut self.diag {
                    Some(subtree) => subtree,
                    None => {
                        self.diag = Some(Box::new(TrieNode::new(tile, generator)));
                        self.diag.as_mut().unwrap()
                    }
                };
                subtree.extend(generator, iter);
            } else if tile.run == self.tile.run && tile.rise == self.tile.rise + 1 {
                unimplemented!("Segments aren't expected to step upwards.");
            } else {
                panic!("Segment is discontinuous!");
            }
        }
    }
}

// TODO: Lenient FOV.
// TODO: Make a lazy-static? I also doubt you will need to increase radius much more than it is already.
#[derive(Debug)]
struct StrictFOV {
    radius: u32,
    origin: TrieNode,
}

impl StrictFOV {
    fn new(radius: u32) -> Self {
        let mut partial = StrictFOV {
            radius: 0,
            origin: TrieNode::new(
                OctantRelative::ignore_octant(0, 0),
                OctantRelative::ignore_octant(0, 0),
            ),
        };

        for _ in 1..radius {
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
            self.origin.extend(generator, a.into_iter().skip(1));
            if let Some(b) = b {
                self.origin.extend(generator, b.into_iter().skip(1));
            }
        }
    }
}

#[cfg(test)]
#[test]
fn new_strict_fov() {
    StrictFOV::new(5);
}
