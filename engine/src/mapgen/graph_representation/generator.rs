use std::num::NonZero;

use rand::Rng;
use rand::distr::Distribution;
use rand::seq::SliceRandom;
use rand_distr::Binomial;
use rand_distr::Uniform;

use crate::mapgen::graph_representation::Branch;

type BranchMutation<RngImpl> = Box<dyn Fn(&mut Branch, &mut RngImpl)>;

struct EdgeCost<RngImpl: Rng>(u8, Vec<BranchMutation<RngImpl>>);
impl<RngImpl: Rng> EdgeCost<RngImpl> {
    fn new_empty() -> Self {
        Self(0, Vec::new())
    }

    fn new_pair(cost: u8, mutation: BranchMutation<RngImpl>) -> Self {
        Self(cost, vec![mutation])
    }

    fn extend(&mut self, other: Self) {
        self.0 += other.0;
        self.1.extend(other.1);
    }

    fn extend_up_to_limit(&mut self, other: Self, cost_limit: u8) -> Result<(), ()> {
        if self.0 + other.0 <= cost_limit {
            self.extend(other);
            Ok(())
        } else {
            Err(())
        }
    }

    fn call_and_extend_up_to_limit(
        &mut self,
        generator: &mut impl FnMut() -> Self,
        cost_limit: u8,
    ) {
        while self.extend_up_to_limit(generator(), cost_limit).is_ok() {}
    }
}

pub struct BranchGenerator {
    // These constraints are always met:
    pub edges: u8,
    pub max_loop_len: u8,
}

impl BranchGenerator {
    pub fn generate(&self, rng: &mut impl Rng) -> Branch {
        let mut total_cost = EdgeCost::new_empty();

        // ~ 5/8 of edges will be part of a polygon (and its spacer)
        let polygon_budget = u8::try_from(
            Binomial::new((self.edges - total_cost.0).into(), 0.625)
                .unwrap()
                .sample(rng),
        )
        .expect("polygon_budget <= self.edges <= u8::MAX");

        total_cost.call_and_extend_up_to_limit(
            &mut || generate_polygon(rng, self.max_loop_len),
            polygon_budget,
        );

        // ~ 1/8 of edges will be straightaways.
        let straightaways = u8::try_from(
            Binomial::new((self.edges - total_cost.0).into(), (0.125) / (1.0 - 0.625))
                .unwrap()
                .sample(rng),
        )
        .expect("straightaways <= self.edges <= u8::MAX");

        total_cost.extend(generate_straightaways(straightaways));

        SliceRandom::shuffle(total_cost.1.as_mut_slice(), rng);

        // ~ 2/8 of edges will be strays/deadends.
        let stray_edges = self.edges - total_cost.0;
        total_cost.extend(generate_stray_edges(stray_edges));

        let mut out = Branch::new();
        for f in total_cost.1 {
            f(&mut out, rng);
        }
        out
    }
}

fn generate_polygon<RngImpl: Rng>(rng: &mut RngImpl, max_loop_len: u8) -> EdgeCost<RngImpl> {
    let polygon_side_distribution = Uniform::new_inclusive(3, max_loop_len).unwrap();
    let sides = polygon_side_distribution.sample(rng);

    let split_unwrapped = u8::try_from(
        Binomial::new(u64::from(sides) - 2, 0.5)
            .unwrap()
            .sample(rng)
            + 1,
    )
    .expect("u8::MAX >= max_loop_len >= sides > sides - 2 + 1 >= sample + 1");

    let split = NonZero::new(split_unwrapped).expect("split_unwrapped is >= 1");
    let other = NonZero::new(sides - split_unwrapped).expect("split_unwrapped is < sides");

    EdgeCost::new_pair(
        sides + 1,
        Box::new(move |branch, _| {
            Branch::extend_join_and_fork(branch, split, other);
        }),
    )
}

fn generate_stray_edges<RngImpl: Rng>(count: u8) -> EdgeCost<RngImpl> {
    let a = |branch: &mut Branch, rng: &mut RngImpl| {
        let candidates = branch.get_candidate_parents().collect::<Vec<_>>();
        let index = rng.random_range(..candidates.len());
        let chosen = candidates.into_iter().nth(index).expect("index is valid");
        branch.add_new_child(chosen);
    };

    let dingus = (0..count)
        .map(|_| Box::new(a) as BranchMutation<RngImpl>)
        .collect();
    EdgeCost::<RngImpl>(count, dingus)
}

fn generate_straightaways<RngImpl: Rng>(count: u8) -> EdgeCost<RngImpl> {
    let a = |branch: &mut Branch, _: &mut RngImpl| {
        branch.extend_source();
    };

    let dingus = (0..count)
        .map(|_| Box::new(a) as BranchMutation<RngImpl>)
        .collect();
    EdgeCost::<RngImpl>(count, dingus)
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    use crate::mapgen::graph_representation::generator::BranchGenerator;

    // #[test]
    // fn pure_function() {
    //     let thing = || {
    //         let mut rng = SmallRng::seed_from_u64(0xb0ba_b0ba);
    //         let branch = BranchGenerator {
    //             edges: 100,
    //             max_loop_len: 100,
    //         }
    //         .generate(&mut rng);

    //         // Assume Debug is also pure, which it probably is.
    //         format!("{branch:?}")
    //     };

    //     assert_eq!(thing(), thing());
    // }

    #[test]
    fn edge_count_correct() {
        let mut rng = SmallRng::from_seed([0u8; 32]);
        let generator = BranchGenerator {
            edges: 20,
            max_loop_len: 8,
        };
        let branch = generator.generate(&mut rng);

        assert_eq!(branch.graph.edge_count(), generator.edges as usize);

        // println!(
        //     "{:?}",
        //     Dot::with_config(&branch.graph, &[Config::EdgeNoLabel, Config::NodeNoLabel])
        // );
        // panic!()
    }
}
