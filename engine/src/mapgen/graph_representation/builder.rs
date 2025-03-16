use std::ops::RangeBounds;

use super::Branch;
use super::CaveSystem;
use super::FlowIn;
use super::FlowOut;
use super::Room;

#[must_use]
pub struct CaveSystemBuilder {
    system: CaveSystem,
}

impl CaveSystemBuilder {
    pub fn build(self) -> CaveSystem {
        self.system
    }

    pub fn generate(
        main_flow: u8,
        other_flow_limit: u8,
        rng: &mut impl FnMut() -> u32,
    ) -> CaveSystem {
        let half = 1.max(main_flow / 2);
        let double = main_flow.saturating_mul(2);

        let mut other_flows: [_; 5] = call_until_max_sum(other_flow_limit, &mut || {
            random_in_range(&(half..=double), rng())
        });
        permute(&mut other_flows, rng());

        let main = BranchBuilder::new_minimal(main_flow)
            .add_edges(10, rng)
            .build();
        let others = other_flows
            .map(|opt| opt.map(|flow| BranchBuilder::new_minimal(flow).add_edges(10, rng).build()));

        CaveSystem { main, others }
    }

    pub fn new_minimal(main_flow: u8) -> Self {
        Self {
            system: CaveSystem {
                main: BranchBuilder::new_minimal(main_flow).build(),
                others: [const { None }; 5],
            },
        }
    }

    pub fn set_main(mut self, main: Branch) -> Self {
        self.system.main = main;
        self
    }

    pub fn set_branch(mut self, branch: Branch, i: usize) -> Self {
        let _ = self.system.others[i].insert(branch);
        self
    }

    pub fn set_branches(mut self, others: [Option<Branch>; 5]) -> Self {
        self.system.others = others;
        self
    }

    pub fn permute_branches(mut self, random: u32) -> Self {
        permute(&mut self.system.others, random);
        self
    }
}

#[must_use]
pub struct BranchBuilder {
    branch: Branch,
}

impl BranchBuilder {
    pub fn build(self) -> Branch {
        self.branch
    }

    pub fn new_minimal(flow: u8) -> Self {
        Self {
            branch: Branch {
                rooms: vec![Room {
                    flow_in: FlowIn::Source,
                    flow_out: FlowOut::None,
                    age: 0,
                    depth: 0,
                }],
                flow,
            },
        }
    }

    pub fn overwrite_flow(mut self, flow: u8) -> Self {
        self.branch.flow = flow;
        self
    }

    pub fn add_edge_and_age(mut self, rng: &mut impl FnMut() -> u32) -> Self {
        let parent = Self::choose_parent(0, &self.branch.rooms, rng);
        let maybe_child = Self::choose_existing_child(&self.branch.rooms, parent, rng);
        Self::connect_parent_to_maybe_child(&mut self.branch.rooms, parent, maybe_child);

        // i am aware this is linear in the number of rooms at worst.
        // so the entire generation is quadratic.
        Self::simulate_age(0, &mut self.branch.rooms);

        self
    }

    pub fn add_edges(mut self, edges: u8, rng: &mut impl FnMut() -> u32) -> Self {
        for _ in 0..edges {
            self = self.add_edge_and_age(rng);
        }
        self
    }

    fn choose_parent(parent: usize, rooms: &[Room], rng: &mut impl FnMut() -> u32) -> usize {
        match rooms[parent].flow_out {
            FlowOut::One(_) | FlowOut::None => {
                if rng() & 1 > 0 {
                    return parent;
                }
            }
            FlowOut::Two(_, _) => (),
        }

        match rooms[parent].flow_out {
            FlowOut::None => parent,
            FlowOut::One(child) | FlowOut::Two(_, child) => Self::choose_parent(child, rooms, rng),
        }
    }

    fn choose_existing_child(
        rooms: &[Room],
        parent: usize,
        rng: &mut impl FnMut() -> u32,
    ) -> Option<usize> {
        let parent_depth = rooms[parent].depth;
        let parent_child = match rooms[parent].flow_out {
            FlowOut::None => None,
            FlowOut::One(x) => Some(x),
            FlowOut::Two(_, _) => unreachable!(),
        };

        let valid_children = rooms.iter().enumerate().filter(|(i, room)| {
            parent_child.is_none_or(|some| some != *i)
                && room.depth > parent_depth
                && !matches!(room.flow_in, FlowIn::Two)
        });

        let valid_indices = valid_children.map(|(i, _)| i);

        // Note that later rooms will be more likely to be selected. This is fine.
        valid_indices.max_by_key(|_| rng())
    }

    fn connect_parent_to_maybe_child(
        rooms: &mut Vec<Room>,
        parent: usize,
        maybe_child: Option<usize>,
    ) {
        let parent_depth = rooms[parent].depth;

        let child_index = maybe_child.unwrap_or(rooms.len());

        rooms[parent].flow_out = match rooms[parent].flow_out {
            FlowOut::None => FlowOut::One(child_index),
            FlowOut::One(x) => FlowOut::Two(x, child_index),
            FlowOut::Two(_, _) => unreachable!(),
        };

        if let Some(some) = maybe_child {
            rooms[some].flow_in = FlowIn::Two;
            some
        } else {
            rooms.push(Room {
                flow_in: FlowIn::One,
                flow_out: FlowOut::None,
                age: 0,
                depth: parent_depth + 1,
            });
            rooms.len() - 1
        };
    }

    fn simulate_age(current: usize, rooms: &mut [Room]) {
        rooms[current].age += 1;
        if let Some(downstream) = match rooms[current].flow_out {
            FlowOut::None => None,
            FlowOut::One(x) | FlowOut::Two(_, x) => Some(x),
        } {
            Self::simulate_age(downstream, rooms);
        }
    }
}

// slightly biased towards lower values with very low probability.
fn random_in_range(range: &(impl RangeBounds<u8> + ExactSizeIterator), random: u32) -> u8 {
    let outputs = u32::try_from(range.len()).expect("this should be at most 256");
    let lt_len = u8::try_from(random % outputs).expect("a % b < b, b is a u8 or exactly 256");

    match range.start_bound() {
        // no overflow since lt_len has max value 255 - x
        std::ops::Bound::Included(x) => *x + lt_len,
        // no overflow since lt_len has max value 254 - x
        std::ops::Bound::Excluded(x) => *x + 1 + lt_len,
        std::ops::Bound::Unbounded => unreachable!("since range implements ExactSizeIterator"),
    }
}

fn call_until_max_sum<const N: usize>(
    maximum: u8,
    sampler: &mut impl FnMut() -> u8,
) -> [Option<u8>; N] {
    let mut out = [None; N];
    let mut sum = 0;
    for slot in &mut out {
        let sample = sampler();
        sum += sample;
        if sum > maximum {
            break;
        }
        *slot = Some(sample);
    }
    out
}

fn permute<T>(slice: &mut [T], mut random_sample: u32) {
    if slice.len() <= 1 {
        return;
    }

    let swap = (random_sample as usize) % slice.len();
    random_sample /= u32::try_from(slice.len()).unwrap();

    slice.swap(slice.len() - 1, swap);

    let last = slice.len() - 1;
    permute(&mut slice[..last], random_sample);
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::ops::RangeBounds;

    use super::call_until_max_sum;
    use super::permute;
    use super::random_in_range;

    #[test]
    fn test_random_in_range() {
        fn for_range(range: &(impl RangeBounds<u8> + ExactSizeIterator)) {
            let mut counts = [0u16; 256];
            // the end range doesn't really matter for testing.
            for i in 0..=300u16 {
                let sample = random_in_range(range, u32::from(i));
                counts[sample as usize] += 1;
            }

            for i in 0..=255u8 {
                assert_eq!(range.contains(&i), counts[i as usize] != 0, "{i}");
            }

            let nonzeros = counts.iter().filter(|c| **c != 0).copied();
            let (min, max) = nonzeros.fold((u16::MAX, u16::MIN), |(min, max), el| {
                (el.min(min), el.max(max))
            });
            assert!(min == max || min + 1 == max);
        }
        for_range(&(5..20));
        for_range(&(0..=255));
        for_range(&(0..=0));
        for_range(&(254..=255));
    }

    #[test]
    fn test_call_until_max_sum() {
        {
            let ones = call_until_max_sum::<50>(42, &mut || 1);
            assert_eq!(ones.iter().flatten().sum::<u8>(), 42u8);
            assert!(ones.iter().all(|opt| opt.is_none_or(|some| some == 1)));
        }
        {
            let mut dingus = 1;
            let mut sampler = || {
                dingus += 2;
                dingus - 2
            };
            let odds_sum_to_square = call_until_max_sum::<20>(99, &mut sampler);
            assert_eq!(odds_sum_to_square.iter().flatten().sum::<u8>(), 81);
            assert!(
                odds_sum_to_square
                    .iter()
                    .all(|opt| opt.is_none_or(|some| some % 2 == 1))
            );
        }
    }

    #[test]
    fn test_permute() {
        let original = [0, 1, 2, 3, 5];
        let mut multiset: HashMap<[i32; 5], u16> = HashMap::new();
        for i in 0..u8::MAX {
            let mut clone = original;
            permute(&mut clone, i.into());
            *multiset.entry(clone).or_default() += 1;
        }

        assert_eq!(multiset.len(), 120, "{multiset:?}");

        let nonzeros = multiset.values().copied();
        let (min, max) = nonzeros.fold((u16::MAX, u16::MIN), |(min, max), el| {
            (el.min(min), el.max(max))
        });
        assert!(min == max || min + 1 == max);
    }
}
