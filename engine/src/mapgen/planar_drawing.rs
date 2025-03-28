#![allow(dead_code, reason = "WIP")]

use std::iter::zip;

use gomez::Problem;
use gomez::Solver;
use gomez::SolverDriver;
use gomez::System;
use gomez::algo::TrustRegion;
use gomez::driver::SolverIterState;
use gomez::nalgebra::DVector;
use gomez::nalgebra::Dyn;
use gomez::nalgebra::IsContiguous;
use gomez::nalgebra::OMatrix;
use gomez::nalgebra::RowDVector;
use gomez::nalgebra::Storage;
use gomez::nalgebra::Vector;
use petgraph::prelude::*;
use petgraph::visit::GetAdjacencyMatrix;
use petgraph::visit::GraphBase;
use petgraph::visit::NodeCount;

#[derive(Debug)]
struct GraphDrawing<T>(T);

impl<T> GraphDrawing<T>
where
    T: GraphBase<NodeId = NodeIndex<u8>> + GetAdjacencyMatrix + NodeCount,
{
    pub fn get_sum_force_magnitude(
        &self,
        xs: DVector<f32>,
        ys: DVector<f32>,
    ) -> (DVector<f32>, DVector<f32>) {
        let ones_like = RowDVector::from_element(xs.len(), 1f32);
        let dxs = {
            let mut repeated_cols = xs * &ones_like;
            let repeated_rows = repeated_cols.transpose();
            repeated_cols -= repeated_rows;
            repeated_cols
        };
        let dys = {
            let mut repeated_cols = ys * &ones_like;
            let repeated_rows = repeated_cols.transpose();
            repeated_cols -= repeated_rows;
            repeated_cols
        };

        let square_distances = dxs.map(|dx| dx.powi(2)) + dys.map(|dy| dy.powi(2));

        let petgraph_adjacency = self.0.adjacency_matrix();
        let adjacency_matrix = OMatrix::<f32, Dyn, Dyn>::from_fn(
            square_distances.nrows(),
            square_distances.ncols(),
            |i, j| {
                let a =
                    self.0
                        .is_adjacent(&petgraph_adjacency, NodeIndex::new(i), NodeIndex::new(j));
                let b =
                    self.0
                        .is_adjacent(&petgraph_adjacency, NodeIndex::new(j), NodeIndex::new(i));
                if a || b { 1f32 } else { 0f32 }
            },
        );

        // F/d = -k
        // Force should pull towards each other. (If the spring is present.)
        let spring_forces = &adjacency_matrix * -1f32;

        // F/d = q1q2/d^3
        // Force should push away from each other. (both q's are the same sign)
        let mut electric_forces = square_distances.map(|f| f.powf(-1.5f32));
        electric_forces.fill_diagonal(0f32);

        let sum_force_xs = (&spring_forces + &electric_forces)
            .component_mul(&dxs)
            .column_sum();
        let sum_force_ys = (spring_forces + electric_forces)
            .component_mul(&dys)
            .column_sum();

        (sum_force_xs, sum_force_ys)
    }

    pub fn to_optimizer_driver(&self) -> GraphDrawingOptimizer<'_, T, TrustRegion<Self>> {
        GraphDrawingOptimizer(
            SolverDriver::builder(self)
                .with_initial(
                    (1..self.0.node_count())
                        .map(|i| u8::try_from(i).expect("node count <= u8::MAX"))
                        .map(f32::from)
                        .chain((2..self.0.node_count()).map(|_| 0f32))
                        .collect(),
                )
                .build(),
        )
    }

    // pub fn into_graph() -> Graph<Position, (), Directed, u8> {
    //     todo!()
    // }
}

// With too many variables, there are many answers that are rotationally symmetric.
// The problem pins the first node at the origin, and the second node on the x axis.
// This reduces the dimensions, but also increases the distance to the answer.
// TODO: Figure out if this is ok.

impl<T> Problem for GraphDrawing<T>
where
    T: NodeCount,
{
    type Field = f32;

    fn domain(&self) -> gomez::Domain<Self::Field> {
        gomez::Domain::unconstrained(self.0.node_count() * 2 - 3)
    }
}

impl<T> System for GraphDrawing<T>
where
    T: GraphBase<NodeId = NodeIndex<u8>> + GetAdjacencyMatrix + NodeCount,
{
    fn eval<Sx, Srx>(
        &self,
        x: &Vector<Self::Field, Dyn, Sx>,
        rx: &mut Vector<Self::Field, Dyn, Srx>,
    ) where
        Sx: Storage<Self::Field, Dyn> + IsContiguous,
        Srx: gomez::nalgebra::StorageMut<Self::Field, Dyn>,
    {
        let (xs, ys) = x.as_slice().split_at(self.0.node_count() - 1);

        // oh boy unnecessary allocations!
        let xs: DVector<f32> = DVector::from_iterator(
            self.0.node_count(),
            [0f32; 1].into_iter().chain(xs.iter().copied()),
        );
        let ys: DVector<f32> = DVector::from_iterator(
            self.0.node_count(),
            [0f32; 2].into_iter().chain(ys.iter().copied()),
        );

        let (sum_forces_x, sum_forces_y) = self.get_sum_force_magnitude(xs, ys);

        for (dest, source) in zip(
            &mut *rx,
            sum_forces_x
                .into_iter()
                .skip(1)
                .chain(sum_forces_y.into_iter().skip(2)),
        ) {
            *dest = *source;
        }

        println!("{rx}");
    }
}

type Positions = [(f32, f32)];

struct GraphDrawingOptimizer<'a, T, A>(SolverDriver<'a, GraphDrawing<T>, A>)
where
    T: GraphBase<NodeId = NodeIndex<u8>> + GetAdjacencyMatrix + NodeCount;

impl<T, A> GraphDrawingOptimizer<'_, T, A>
where
    T: GraphBase<NodeId = NodeIndex<u8>> + GetAdjacencyMatrix + NodeCount,
    A: Solver<GraphDrawing<T>>,
{
    fn find<C>(&mut self, stop: C) -> Result<(Box<Positions>, f32), A::Error>
    where
        C: Fn(SolverIterState<'_, GraphDrawing<T>>) -> bool,
    {
        self.0.find(stop).map(|(x, fx)| (Self::vec_to_pairs(x), fx))
    }

    fn x(&self) -> Box<Positions> {
        Self::vec_to_pairs(self.0.x())
    }

    fn vec_to_pairs(vec: &[f32]) -> Box<Positions> {
        let (xs, ys) = vec.split_at((vec.len() + 3) / 2 - 1);
        zip(
            [0f32; 1].into_iter().chain(xs.iter().copied()),
            [0f32; 2].into_iter().chain(ys.iter().copied()),
        )
        .collect::<Box<_>>()
    }
}

#[cfg(test)]
mod test {
    use crate::mapgen::graph_representation::Branch;
    use crate::mapgen::planar_drawing::GraphDrawing;

    macro_rules! assert_close {
        ($p: expr, $q: expr) => {
            let square_dist = ($p.0 - $q.0).powi(2) + ($p.1 - $q.1).powi(2);

            assert!(
                square_dist < 1e-8,
                "{:?}\n{:?}\nSquareDistance: {}",
                $p,
                $q,
                square_dist
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

        // Since there is only one variable, the x coord of the second node, we can expect to solve this quickly.
        // The spring force (x) should equal the electric force (1/x^2).
        //    x = 1/x^2
        // => x^3 = 1
        // => x = 1
        assert_close!(x[0], (0f32, 0f32));
        assert_close!(x[1], (1f32, 0f32));
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

        assert_close!(x[1], (expected_distance, 0f32));
        assert_close!(x[2], (2f32 * expected_distance, 0f32));
    }
}
