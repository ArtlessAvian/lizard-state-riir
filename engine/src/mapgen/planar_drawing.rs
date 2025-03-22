#![expect(dead_code, reason = "WIP")]

use std::iter::zip;

use gomez::Problem;
use gomez::Solver;
use gomez::SolverDriver;
use gomez::System;
use gomez::algo::TrustRegion;
use gomez::driver::SolverIterState;
use gomez::nalgebra::Dyn;
use gomez::nalgebra::IsContiguous;
use gomez::nalgebra::Storage;
use gomez::nalgebra::Vector;
use petgraph::prelude::*;
use petgraph::visit::IntoEdges;
use petgraph::visit::NodeCount;

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

impl Position {
    // Notably, f(0) = (0, 0) and f(1) = (_, 0)
    fn new_injective(i: u8) -> Self {
        let i = f32::from(i);
        Position {
            x: i * 100f32,
            y: 0f32,
        }
    }

    fn square_dist(self, other: Self) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    fn normalized_vector_to(self, other: Self) -> (f32, f32) {
        let len = self.square_dist(other).sqrt();
        ((other.x - self.x) / len, (other.y - self.y) / len)
    }

    fn into_iter(self) -> impl Iterator<Item = f32> {
        [self.x, self.y].into_iter()
    }
}

#[derive(Debug)]
struct GraphDrawing<T>(T);

impl<T> GraphDrawing<T>
where
    T: IntoEdges<NodeId = NodeIndex<u8>> + NodeCount,
{
    fn suggested_initial_positions(&self) -> impl Iterator<Item = Position> {
        let node_count = u8::try_from(self.0.node_count()).expect("structure indices are u8s");
        (0..node_count).map(Position::new_injective)
    }

    fn get_energy(&self, positions: &[Position]) -> f32 {
        let mut total = 0f32;

        for (pi, p) in positions.iter().enumerate() {
            for (qi, q) in positions.iter().enumerate() {
                if pi <= qi {
                    continue;
                }
                total += 1f32 / p.square_dist(*q).sqrt();
            }
        }

        for edge in self.0.edge_references() {
            let p = positions[edge.source().index()];
            let q = positions[edge.target().index()];
            total += p.square_dist(q);
        }

        total
    }

    fn get_sum_forces(&self, positions: &[Position]) -> Vec<(f32, f32)> {
        let mut out = positions.iter().map(|_| (0f32, 0f32)).collect::<Vec<_>>();

        for (pi, p) in positions.iter().enumerate() {
            for (qi, q) in positions.iter().enumerate() {
                if pi == qi {
                    continue;
                }
                let (x, y) = p.normalized_vector_to(*q);
                out[pi].0 -= 1f32 / p.square_dist(*q) * x;
                out[pi].1 -= 1f32 / p.square_dist(*q) * y;
            }
        }

        for edge in self.0.edge_references() {
            let pi = edge.source().index();
            let p = positions[pi];
            let qi = edge.target().index();
            let q = positions[qi];

            let (x, y) = p.normalized_vector_to(q);
            out[pi].0 += p.square_dist(q).sqrt() * x / 1f32;
            out[pi].1 += p.square_dist(q).sqrt() * y / 1f32;
            out[qi].0 -= p.square_dist(q).sqrt() * x / 1f32;
            out[qi].1 -= p.square_dist(q).sqrt() * y / 1f32;
        }

        out
    }

    pub fn positions_to_unpinned_elements(
        positions: impl IntoIterator<Item = Position>,
    ) -> impl Iterator<Item = f32> {
        let mut elements = positions.into_iter().flat_map(Position::into_iter);
        elements.next();
        elements.next();
        let node_1_x = elements.next();
        elements.next();
        node_1_x.into_iter().chain(elements)
    }

    pub fn unpinned_elements_to_positions(
        mut unpinned: impl Iterator<Item = f32>,
    ) -> Vec<Position> {
        let Some(node_1_x) = unpinned.next() else {
            return [Position { x: 0f32, y: 0f32 }].to_vec();
        };

        let elements = [0f32, 0f32, node_1_x, 0f32]
            .into_iter()
            .chain(unpinned)
            .collect::<Box<_>>();

        elements
            .chunks_exact(2)
            .map(|chunk| Position {
                x: chunk[0],
                y: chunk[1],
            })
            .collect()
    }

    pub fn to_optimizer_driver(&self) -> GraphDrawingOptimizer<'_, T, TrustRegion<Self>> {
        let unpinned = Self::positions_to_unpinned_elements(self.suggested_initial_positions());

        GraphDrawingOptimizer(
            SolverDriver::builder(self)
                .with_initial(unpinned.collect())
                .build(),
        )
    }

    pub fn into_graph() -> Graph<Position, (), Directed, u8> {
        todo!()
    }
}

impl<T> Clone for GraphDrawing<T>
where
    T: Copy, // INTENTIONAL
{
    /// Clone by reference
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Problem for GraphDrawing<T>
where
    T: IntoEdges<NodeId = NodeIndex<u8>> + NodeCount,
{
    type Field = f32;

    fn domain(&self) -> gomez::Domain<Self::Field> {
        gomez::Domain::unconstrained(
            Self::positions_to_unpinned_elements(self.suggested_initial_positions()).count(),
        )
    }
}

// impl<T> Function for GraphDrawing<T>
// where
//     T: IntoEdges<NodeId = NodeIndex<u8>> + NodeCount,
// {
//     fn apply<Sx>(&self, x: &Vector<Self::Field, Dyn, Sx>) -> Self::Field
//     where
//         Sx: Storage<Self::Field, Dyn> + IsContiguous,
//     {
//         let positions = Self::unpinned_elements_to_positions(x.iter().copied());
//         self.get_energy(&positions)
//     }
// }

impl<T> System for GraphDrawing<T>
where
    T: IntoEdges<NodeId = NodeIndex<u8>> + NodeCount,
{
    fn eval<Sx, Srx>(
        &self,
        x: &Vector<Self::Field, Dyn, Sx>,
        rx: &mut Vector<Self::Field, Dyn, Srx>,
    ) where
        Sx: Storage<Self::Field, Dyn> + IsContiguous,
        Srx: gomez::nalgebra::StorageMut<Self::Field, Dyn>,
    {
        let positions = Self::unpinned_elements_to_positions(x.iter().copied());
        let sum_forces = self.get_sum_forces(&positions);
        let mut sum_forces_components = sum_forces.into_iter().flat_map(|a| [a.0, a.1].into_iter());
        sum_forces_components.next();
        sum_forces_components.next();
        let node_1_x = sum_forces_components.next();
        sum_forces_components.next();

        for (dest, source) in zip(rx, node_1_x.into_iter().chain(sum_forces_components)) {
            *dest = source;
        }
    }
}

struct GraphDrawingOptimizer<'a, T, A>(SolverDriver<'a, GraphDrawing<T>, A>)
where
    T: IntoEdges<NodeId = NodeIndex<u8>> + NodeCount;

impl<T, A> GraphDrawingOptimizer<'_, T, A>
where
    T: IntoEdges<NodeId = NodeIndex<u8>> + NodeCount,
    A: Solver<GraphDrawing<T>>,
{
    fn find<C>(&mut self, stop: C) -> Result<(Vec<Position>, f32), A::Error>
    where
        C: Fn(SolverIterState<'_, GraphDrawing<T>>) -> bool,
    {
        self.0.find(stop).map(|(x, fx)| {
            (
                GraphDrawing::<T>::unpinned_elements_to_positions(x.iter().copied()),
                fx,
            )
        })
    }

    fn x(&self) -> Vec<Position> {
        GraphDrawing::<T>::unpinned_elements_to_positions(self.0.x().iter().copied())
    }

    // fn fx(&self) -> f32 {
    //     self.0.fx()
    // }
}

#[cfg(test)]
mod test {
    use crate::mapgen::graph_representation::Branch;
    use crate::mapgen::planar_drawing::GraphDrawing;
    use crate::mapgen::planar_drawing::Position;

    macro_rules! assert_close {
        ($p: expr, $q: expr) => {
            assert!(
                $p.square_dist($q) < 1e-8,
                "{:?}\n{:?}\nSquareDistance: {}",
                $p,
                $q,
                $p.square_dist($q)
            );
        };
    }

    #[test]
    fn line_graph_of_two() {
        let mut branch = Branch::new();
        branch.extend_source();

        let drawing = GraphDrawing(branch.graph.inner());
        let mut driver = drawing.to_optimizer_driver();
        let x = driver
            .find(|state| {
                dbg!(state.x());
                state.iter() > 100
            })
            .map_or_else(|_| driver.x(), |x| x.0);

        assert_close!(x[0], Position { x: 0f32, y: 0f32 });
        assert_close!(x[1], Position { x: 1f32, y: 0f32 });
    }

    #[test]
    fn line_graph_of_three() {
        let mut branch = Branch::new();
        branch.extend_source();
        branch.extend_source();

        let drawing = GraphDrawing(branch.graph.inner());
        let mut driver = drawing.to_optimizer_driver();
        let x = driver
            .find(|state| state.iter() > 100)
            .map_or_else(|_| driver.x(), |x| x.0);

        // The solution should be colinear.
        // We can assume the line is aligned with the x axis. The pinned coordinates make this so.
        // We assume the middle node is in the middle. (There is another solution where it isn't.)
        // The sum forces of the middle node is 0 when both ends are opposite at the same distance.
        // The sum force of an end node is one spring and two electrical forces.
        // We want d = 1/d^2 + 1/(2d)^2 => d^3 = 5/4.
        let expected_distance = (5f32 / 4f32).powf(1f32 / 3f32);

        assert_close!(
            x[1],
            Position {
                x: expected_distance,
                y: 0f32,
            }
        );
        assert_close!(
            x[2],
            Position {
                x: 2f32 * expected_distance,
                y: 0f32,
            }
        );
    }
}
