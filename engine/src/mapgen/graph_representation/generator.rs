use std::num::NonZero;

use rand::Rng;
use rand::distr::Distribution;
use rand::seq::SliceRandom;
use rand_distr::Zipf;

use crate::mapgen::graph_representation::Branch;

pub struct BranchGenerator {
    // These constraints are always met:
    edges: u8,
    max_loop_len: u8,
}

impl BranchGenerator {
    pub fn generate(&self, rng: &mut impl Rng) -> Branch {
        let mut out = Branch::new();

        let mut deferred_calls = Vec::<Box<dyn Fn(&mut Branch)>>::new();

        let mut edge_budget = self.edges;

        {
            // 80% of edges will be part of a polygon (and its spacer)
            let binomial = rand_distr::Binomial::new(edge_budget.into(), 0.8).unwrap();
            let polygon_side_distribution =
                Zipf::new(f32::from(self.max_loop_len - 2), 1f32).unwrap();

            let polygon_budget = u8::try_from(binomial.sample(rng))
                .expect("max value is edge_budget, which is a u8");
            let mut polygon_edges = 0;
            loop {
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "zipf distribution returns 1-10"
                )]
                let sides_unoffset = polygon_side_distribution.sample(rng) as u8; // [1, max]
                let sides = sides_unoffset + 2; // [3, max]

                if polygon_edges + sides + 1 > polygon_budget {
                    break;
                }

                let split_unwrapped = rng.random_range(1..sides);
                let split = NonZero::new(split_unwrapped).expect("split_unwrapped is >= 1");
                let other =
                    NonZero::new(sides - split_unwrapped).expect("split_unwrapped is < sides");
                deferred_calls.push(Box::new(move |x| {
                    Branch::extend_join_and_fork(x, split, other);
                }));
                polygon_edges += sides + 1;
            }

            edge_budget -= polygon_edges;
        }

        for _ in 0..edge_budget {
            deferred_calls.push(Box::new(Branch::extend_source));
        }

        SliceRandom::shuffle(deferred_calls.as_mut_slice(), rng);
        for f in deferred_calls {
            f(&mut out);
        }

        out
    }
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    use crate::mapgen::graph_representation::generator::BranchGenerator;

    #[test]
    fn pure_function() {
        let thing = || {
            let mut rng = SmallRng::seed_from_u64(0xb0ba_b0ba);
            let branch = BranchGenerator {
                edges: 10,
                max_loop_len: 10,
            }
            .generate(&mut rng);

            // Assume Debug is also pure, which it probably is.
            format!("{branch:?}")
        };

        assert_eq!(thing(), thing());
    }

    #[test]
    fn edge_count_correct() {
        let mut rng = SmallRng::from_seed([0u8; 32]);
        let generator = BranchGenerator {
            edges: 20,
            max_loop_len: 5,
        };
        let branch = generator.generate(&mut rng);

        assert_eq!(branch.graph.edge_count(), generator.edges as usize);
    }
}
