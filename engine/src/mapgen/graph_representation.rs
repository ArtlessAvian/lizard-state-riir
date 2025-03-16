pub mod builder;

/// The setting of the game, represented as a graph.
/// An underground body of water, where a few streams collect.
///
/// Nodes are rooms. Edges are hallways. The graph is planar, since we don't want crossing edges.
///
/// We want to generate graphs that are "fun" and have nice properties.
/// Graphs that are easy to tour are presumably "fun."
///
/// We don't really care to generate every possible graph matching the requirements.
#[derive(Debug)]
#[must_use]
pub struct CaveSystem {
    main: Branch,
    others: [Option<Branch>; 5],
}

impl CaveSystem {
    #[must_use]
    pub fn total_flow(&self) -> u16 {
        u16::from(self.main.flow + self.others.iter().flatten().map(|x| x.flow).sum::<u8>())
    }
}

/// One of many rivers leading to the reservoir.
#[derive(Debug)]
#[must_use]
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
#[must_use]
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
    use super::Branch;
    use super::FlowIn;
    use super::FlowOut;
    use super::builder::BranchBuilder;
    use super::builder::CaveSystemBuilder;

    fn generate_line() -> Branch {
        BranchBuilder::new_minimal(10)
            .add_edges(10, &mut || u32::MIN)
            .build()
    }

    fn generate_max_connections() -> Branch {
        // also min nodes ig.
        BranchBuilder::new_minimal(10)
            .add_edges(10, &mut || u32::MAX)
            .build()
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
        BranchBuilder::new_minimal(10)
            .add_edges(10, &mut low_effort_rng())
            .build()
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
    fn cave_system_smoke_test() {
        let system = CaveSystemBuilder::generate(20, u8::MAX, &mut low_effort_rng());
        assert_eq!(system.main.flow, 20);
    }
}
