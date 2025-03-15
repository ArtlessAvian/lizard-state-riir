use std::ops::RangeBounds;

/// The setting of the game, represented as a graph.
/// An underground body of water, where a few streams collect.
///
/// Nodes are rooms. Edges are hallways. The graph is planar, since we don't want crossing edges.
///
/// We want to generate graphs that are "fun" and have nice properties.
/// Graphs that are easy to tour are presumably "fun."
///
/// We don't really care to generate every possible graph matching the requirements.
pub struct CaveSystem {
    main: Branch,
    others: [Option<Branch>; 5],
}

impl CaveSystem {
    pub fn generate(
        main_flow: u8,
        max_total_other_flow: u8,
        rng: &mut impl FnMut() -> u32,
    ) -> Self {
        // We will call generate_with_all_flows.
        let mut other_flows = [const { None }; 5];

        let half = 1u8.max(main_flow / 2);
        let double = main_flow.saturating_mul(2);

        let mut remaining_flow = max_total_other_flow;
        for opt_room in &mut other_flows {
            let sample = random_in_range(&(half..=double), rng());
            if remaining_flow < sample {
                break;
            }
            remaining_flow -= sample;
            *opt_room = Some(sample);
        }
        // If there's still remaining flow, oh well!

        // Randomly permute the 5 branches. There's 120 permutations.
        // 2^32 possible u32s do not divide evenly into 120. Does it really matter?
        let mut permutation = (rng() % (5 * 4 * 3 * 2)) as usize;
        for i in (0..5).rev() {
            let before = permutation % (i + 1);
            permutation /= i + 1; // floor is expected
            other_flows.swap(i, before);
            // We will never touch other[n] for n >= i after this.
        }

        CaveSystem::generate_with_all_flows(main_flow, other_flows, rng)
    }

    pub fn generate_with_all_flows(
        main_flow: u8,
        other_flows: [Option<u8>; 5],
        rng: &mut impl FnMut() -> u32,
    ) -> Self {
        let main = Branch::generate(main_flow, rng);
        let others = other_flows.map(|opt| opt.map(|some| Branch::generate(some, rng)));
        CaveSystem { main, others }
    }

    // pub fn generate_with_premade_branches(
    //     main_flow: u8,
    //     other_flows: [Option<Either<u8, Branch>>],
    //     rng: &mut impl FnMut() -> u32,
    // ) -> Self {
    // }

    #[must_use]
    pub fn total_flow(&self) -> u16 {
        u16::from(self.main.flow + self.others.iter().flatten().map(|x| x.flow).sum::<u8>())
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

/// One of many rivers leading to the reservoir.
#[derive(Debug)]
pub struct Branch {
    rooms: Vec<Room>,
    flow: u8,
}

impl Branch {
    pub fn generate(flow: u8, rng: &mut impl FnMut() -> u32) -> Self {
        let mut rooms = Vec::new();
        rooms.push(Room {
            flow_in: FlowIn::Source,
            flow_out: FlowOut::None,
            age: 0,
            depth: 0,
        });

        // Each loop adds one edge.
        for _ in 0..10 {
            let parent = Self::choose_parent(0, &rooms, rng);
            let maybe_child = Self::choose_existing_child(&rooms, parent, rng);
            Self::connect_parent_to_maybe_child(&mut rooms, parent, maybe_child);

            // i am aware this is linear in the number of rooms at worst.
            // so the entire generation is quadratic.
            Self::simulate_age(0, &mut rooms);
        }

        Branch { rooms, flow }
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

    #[must_use]
    pub fn dfs_with_duplicates(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack = [0usize].to_vec();

        while let Some(some) = stack.pop() {
            out.push(some);
            match self.rooms[some].flow_out {
                FlowOut::None => {}
                FlowOut::One(x) => stack.push(x),
                FlowOut::Two(x, y) => {
                    stack.push(x);
                    stack.push(y);
                }
            }
        }
        out
    }

    #[must_use]
    pub fn river_path(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let mut current = 0usize;
        loop {
            out.push(current);
            match self.rooms[current].flow_out {
                FlowOut::None => break,
                FlowOut::One(x) | FlowOut::Two(_, x) => current = x,
            }
        }
        out
    }
}

#[derive(Debug)]
struct Room {
    flow_in: FlowIn,
    flow_out: FlowOut,
    depth: u8,
    age: u8,
}

#[derive(Debug, PartialEq, Eq)]
enum FlowIn {
    Source,
    One,
    Two,
}

#[derive(Debug)]
enum FlowOut {
    None,
    One(usize),
    Two(usize, usize),
}

#[cfg(test)]
mod test {
    use std::ops::RangeBounds;

    use super::Branch;
    use super::CaveSystem;
    use super::FlowOut;
    use super::random_in_range;
    use crate::mapgen::graph_representation::FlowIn;

    fn generate_line() -> Branch {
        Branch::generate(10, &mut || u32::MIN)
    }

    fn generate_max_connections() -> Branch {
        // also min nodes ig.
        Branch::generate(10, &mut || u32::MAX)
    }

    fn low_effort_rng() -> impl FnMut() -> u32 {
        // whatever.
        let mut yeag: u32 = 0xB0BA_CAFE_u32;
        move || {
            yeag ^= yeag.rotate_left(1);
            yeag
        }
    }

    fn generate_arbitrary() -> Branch {
        Branch::generate(10, &mut low_effort_rng())
    }

    #[test]
    fn line_is_line() {
        let line = generate_line();

        let all_vec = (0..line.rooms.len()).collect::<Vec<usize>>();
        assert_eq!(line.dfs_with_duplicates(), all_vec);
        assert_eq!(line.river_path(), all_vec);
    }

    #[test]
    fn count_edges() {
        fn for_branch(branch: &Branch) {
            let mut total_in = 0;
            let mut total_out = 0;

            for room in &branch.rooms {
                total_in += match room.flow_in {
                    FlowIn::Source => 0,
                    FlowIn::One => 1,
                    FlowIn::Two => 2,
                };

                total_out += match room.flow_out {
                    FlowOut::None => 0,
                    FlowOut::One(_) => 1,
                    FlowOut::Two(_, _) => 2,
                }
            }
            assert_eq!(total_in, total_out);
        }
        for_branch(&generate_line());
        for_branch(&generate_max_connections());
        for_branch(&generate_arbitrary());
    }

    #[test]
    fn connected() {
        fn for_branch(branch: &Branch) {
            let dfs = branch.dfs_with_duplicates();
            for i in 0..branch.rooms.len() {
                assert!(
                    dfs.contains(&i),
                    "Branch with {dfs:?} does not contain {i}. \n Branch: {branch:?}"
                );
            }
        }
        for_branch(&generate_line());
        for_branch(&generate_max_connections());
        for_branch(&generate_arbitrary());
    }

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
            // let (min, max) = nonzeros.reduce(|(min, max), |)
            assert!(min == max || min + 1 == max);
        }
        for_range(&(5..20));
        for_range(&(0..=255));
        for_range(&(0..=0));
        for_range(&(254..=255));
    }

    #[test]
    fn cave_system_only_main() {
        let system = CaveSystem::generate(10, 0, &mut || u32::MIN);
        assert!(system.others.iter().all(Option::is_none));

        // This implies DFS never saw a fork, so the graph is a line.
        assert_eq!(
            system.main.dfs_with_duplicates().len(),
            system.main.rooms.len(),
        );
    }

    #[test]
    fn cave_system_with_all_flows() {
        let system = CaveSystem::generate_with_all_flows(
            10,
            [None, Some(20), None, Some(30), None],
            &mut || u32::MIN,
        );
        assert_eq!(system.main.flow, 10);

        assert!(system.others[0].is_none());
        assert!(system.others[2].is_none());
        assert!(system.others[4].is_none());

        assert_eq!(system.others[1].as_ref().unwrap().flow, 20);
        assert_eq!(system.others[3].as_ref().unwrap().flow, 30);
    }

    #[test]
    fn cave_system_generated_flows() {
        let system = CaveSystem::generate(10, u8::MAX, &mut || u32::MIN);

        for other in system.others {
            let some = other.unwrap();
            assert_eq!(some.flow, 5);
        }
    }
}
