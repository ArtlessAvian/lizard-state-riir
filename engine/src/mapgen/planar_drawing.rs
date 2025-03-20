#![expect(dead_code, reason = "WIP")]

use petgraph::prelude::*;
use petgraph::visit::IntoNodeReferences;

struct Position {
    x: f32,
    y: f32,
}

impl Position {
    fn square_dist(&self, other: &Self) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}

struct GraphDrawing<'a, N, E> {
    structure: &'a Graph<N, E, Directed, u8>,
    positions: Graph<Position, (), Directed, u8>,
}

impl<'a, N, E> GraphDrawing<'a, N, E> {
    fn from_other(structure: &'a Graph<N, E, Directed, u8>) -> GraphDrawing<'a, N, E> {
        // No colinear points.
        let strictly_convex_function = |i: u8| Position {
            x: f32::from(i),
            y: f32::from(i).sqrt(),
        };

        #[expect(
            clippy::cast_possible_truncation,
            reason = "index is u8, but was widened to usize"
        )]
        let positions = structure.map(|i, _| strictly_convex_function(i.index() as u8), |_, _| ());

        Self {
            structure,
            positions,
        }
    }

    // TODO: Minimize this function!
    fn get_energy(&self) -> f32 {
        let mut total = 0f32;

        for node in self.positions.node_references() {
            for other in self.positions.node_references() {
                if node.0 == other.0 {
                    continue;
                }
                let square_dist = node.1.square_dist(other.1);
                if self.positions.contains_edge(node.0, other.0) {
                    total += square_dist;
                }
                total += 1f32 / square_dist;
            }
        }

        total
    }
}

#[cfg(test)]
mod test {
    use crate::mapgen::graph_representation::Branch;
    use crate::mapgen::planar_drawing::GraphDrawing;

    #[test]
    fn dingus() {
        let mut branch = Branch::new();
        branch.extend_source();

        let drawing = GraphDrawing::from_other(&branch.graph);
        drawing.get_energy();

        // minimum should be 2, unless i changed the energy function
    }
}
