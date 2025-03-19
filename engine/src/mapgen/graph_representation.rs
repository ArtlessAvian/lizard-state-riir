use traversal::CaveSystemNode;

pub mod builder;
pub mod traversal;

/// The setting of the game, represented as a graph.
/// An underground body of water, where a few streams collect.
///
/// Nodes are rooms. Edges are hallways. The graph is planar, since we don't want crossing edges.
///
/// We want to generate graphs that are "fun" and have nice properties.
/// Graphs that are easy to tour are presumably "fun."
///
/// We don't really care to generate every possible graph matching the requirements.
#[derive(Debug, Hash)]
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

    pub fn get_reservoir(&self) -> CaveSystemNode {
        CaveSystemNode::Reservoir { system: self }
    }

    pub fn get_main_source(&self) -> CaveSystemNode {
        CaveSystemNode::InRoom {
            system: self,
            branch: &self.main,
            room: self.main.rooms.first().unwrap(),
        }
    }

    #[must_use]
    pub fn get_all_nodes(&self) -> Vec<CaveSystemNode> {
        let mut out = Vec::new();
        out.push(self.get_reservoir());
        for room in &self.main.rooms {
            out.push(CaveSystemNode::InRoom {
                system: self,
                branch: &self.main,
                room,
            });
        }
        for other in self.others.iter().flatten() {
            for room in &other.rooms {
                out.push(CaveSystemNode::InRoom {
                    system: self,
                    branch: other,
                    room,
                });
            }
        }
        out
    }

    #[must_use]
    /// Lists all edges twice. if (a, b) is present, (b, a) is present.
    pub fn get_all_edges(&self) -> Vec<(CaveSystemNode, CaveSystemNode)> {
        let mut out = Vec::new();
        for from in self.get_all_nodes() {
            for to in from.get_neighbors() {
                if !out.contains(&(from, to)) {
                    out.push((from, to));
                }
            }
        }
        out
    }
}

/// One of many rivers leading to the reservoir.
///
/// Technically directed using `FlowOut`, but rooms also know which rooms connect to them.
#[derive(Debug, Hash)]
#[must_use]
pub struct Branch {
    rooms: Vec<Room>,
    flow: u8,
}

impl Branch {
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

    #[must_use]
    pub fn get_river_end(&self) -> usize {
        self.rooms.len() - 1
    }
}

#[derive(Debug, Hash)]
#[must_use]
pub struct Room {
    flow_in: FlowIn,
    flow_out: FlowOut,
    depth: u8,
    age: u8,
}

#[derive(Debug, Hash)]
enum FlowIn {
    Source,
    One(usize),
    Two(usize, usize),
}

#[derive(Debug, Hash)]
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
                    FlowIn::One(_) => 1,
                    FlowIn::Two(_, _) => 2,
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
    fn valid_back_edges() {
        fn for_branch(branch: &Branch) {
            for (parent, parent_room) in branch.rooms.iter().enumerate() {
                match parent_room.flow_out {
                    FlowOut::None => (),
                    FlowOut::One(left) => match branch.rooms[left].flow_in {
                        FlowIn::Source => unreachable!(),
                        FlowIn::One(left_parent) => assert_eq!(parent, left_parent),
                        FlowIn::Two(left_parent, right_parent) => {
                            assert!(parent == left_parent || parent == right_parent);
                        }
                    },
                    FlowOut::Two(left, right) => {
                        match branch.rooms[left].flow_in {
                            FlowIn::Source => unreachable!(),
                            FlowIn::One(left_parent) => assert_eq!(parent, left_parent),
                            FlowIn::Two(left_parent, right_parent) => {
                                assert!(parent == left_parent || parent == right_parent);
                            }
                        }
                        match branch.rooms[right].flow_in {
                            FlowIn::Source => unreachable!(),
                            FlowIn::One(left_parent) => assert_eq!(parent, left_parent),
                            FlowIn::Two(left_parent, right_parent) => {
                                assert!(parent == left_parent || parent == right_parent);
                            }
                        }
                    }
                }
            }
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
