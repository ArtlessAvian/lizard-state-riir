#![allow(dead_code, reason = "WIP")]

pub mod generator;

use std::num::NonZero;

use petgraph::Graph;
use petgraph::prelude::*;

/// The setting of the game, represented as a graph.
/// An underground body of water, where a few streams collect.
///
/// # Planarity
/// This represents a star graph. Between 1 and 6 nodes connect to a central node.
/// Each outer node is then replaced with another planar graph, the `Branch`
pub struct CaveSystem {
    main: Branch,
    others: [Option<Branch>; 5],
}

impl CaveSystem {}

#[derive(Debug)]
pub struct InDegreeZero(NodeIndex<u8>);

/// One of many rivers leading to the reservoir.
///
/// # Planarity
/// The branch can fork and join with itself. Any join must be with the fork right before.
/// The indegree is at most 2. The outdegree is at most 2.
/// The sum indegree and outdegree is at most 3.
/// One way to imagine this a line of several polygons, with one edge between adjacent polygons. Stray edges are also allowed.
/// This is how the construction works also.
#[derive(Debug)]
pub struct Branch {
    // A graph with at least one node.
    // Every node must have exactly ONE outgoing edge marked true.
    pub graph: Graph<Room, Hall, Directed, u8>,
    source: InDegreeZero,
    sink: NodeIndex<u8>,
}

#[derive(Debug)]
pub struct Room {
    has_flow: bool,
    // Flavor number. The flow into a room minus the flow absorbed is the flow out of the room.
    flow_seen: u8,
    flow_absorbed: u8,
}

impl Room {
    fn new(has_flow: bool) -> Self {
        Self {
            has_flow,
            flow_seen: 0,
            flow_absorbed: 0,
        }
    }
}

#[derive(Debug)]
pub struct Hall {
    has_flow: bool,
    // An arbitrary number.
    flow_seen: u8,
}

impl Hall {
    fn new(has_flow: bool) -> Self {
        Self {
            has_flow,
            flow_seen: 0,
        }
    }
}

impl Default for Branch {
    fn default() -> Self {
        Self::new()
    }
}

impl Branch {
    #[must_use]
    pub fn new() -> Self {
        let mut graph = Graph::<_, _, Directed, u8>::default();
        let sink = graph.add_node(Room {
            has_flow: true,
            flow_seen: 0,
            flow_absorbed: 0,
        });
        let source = InDegreeZero(sink);
        Self {
            graph,
            source,
            sink,
        }
    }

    pub fn extend_source(&mut self) {
        let existing_child = self.source.0;
        let new_parent = self.graph.add_node(Room {
            has_flow: true,
            flow_seen: 0,
            flow_absorbed: 0,
        });
        self.graph.add_edge(
            new_parent,
            existing_child,
            Hall {
                has_flow: true,
                flow_seen: 0,
            },
        );
        self.source = InDegreeZero(new_parent);
    }

    pub fn extend_join_and_fork(&mut self, wet_edges: NonZero<u8>, dry_edges: NonZero<u8>) {
        let shared_join = self.source.0;
        let mut wet_destination = shared_join;
        let mut dry_destination = shared_join;
        for _ in 0..(u8::from(wet_edges) - 1u8) {
            let new_parent = self.graph.add_node(Room {
                has_flow: true,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph
                .add_edge(new_parent, wet_destination, Hall::new(true));
            wet_destination = new_parent;
        }
        for _ in 0..(u8::from(dry_edges) - 1) {
            let new_parent = self.graph.add_node(Room {
                has_flow: false,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph
                .add_edge(new_parent, dry_destination, Hall::new(true));
            dry_destination = new_parent;
        }

        let shared_fork = self.graph.add_node(Room {
            has_flow: true,
            flow_seen: 0,
            flow_absorbed: 0,
        });
        self.graph
            .add_edge(shared_fork, wet_destination, Hall::new(true));
        self.graph
            .add_edge(shared_fork, dry_destination, Hall::new(false));

        let spacer = self.graph.add_node(Room {
            has_flow: true,
            flow_seen: 0,
            flow_absorbed: 0,
        });
        self.graph.add_edge(spacer, shared_fork, Hall::new(true));
        self.source = InDegreeZero(spacer);
    }

    pub fn add_new_child(&mut self, parent: NodeIndex<u8>) -> Option<NodeIndex<u8>> {
        if self.graph.neighbors_directed(parent, Outgoing).count() >= 2 {
            return None;
        }

        if self.sink == parent {
            let new_sink = self.graph.add_node(Room {
                has_flow: true,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph.add_edge(parent, new_sink, Hall::new(true));
            self.sink = new_sink;
            Some(new_sink)
        } else {
            let new_child = self.graph.add_node(Room {
                has_flow: false,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph.add_edge(parent, new_child, Hall::new(false));
            Some(new_child)
        }
    }

    pub fn add_new_parent(&mut self, child: NodeIndex<u8>) -> Option<NodeIndex<u8>> {
        if self.graph.neighbors_directed(child, Incoming).count() >= 2 {
            return None;
        }

        if child == self.source.0 {
            let new_source = self.graph.add_node(Room {
                has_flow: true,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph.add_edge(new_source, child, Hall::new(true));

            self.source.0 = new_source;
            Some(new_source)
        } else {
            let new_parent = self.graph.add_node(Room {
                has_flow: false,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph.add_edge(new_parent, child, Hall::new(true));

            Some(new_parent)
        }
    }

    #[must_use]
    pub fn get_river_nodes(&self) -> Vec<NodeIndex<u8>> {
        let mut out = Vec::new();
        let mut maybe = Some(self.source.0);
        while let Some(current) = maybe {
            out.push(current);

            maybe = self
                .graph
                .edges_directed(current, Direction::Outgoing)
                .find_map(|edge| edge.weight().has_flow.then(|| edge.target()));
        }
        out
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZero;

    use petgraph::prelude::*;
    use petgraph::visit::IntoNodeReferences;

    use super::Branch;

    fn assert_valid(branch: &Branch) {
        // The source as given by the branch has no incoming edges.
        assert!(branch.graph.node_weight(branch.source.0).is_some());
        assert_eq!(
            branch
                .graph
                .neighbors_directed(branch.source.0, Incoming)
                .count(),
            0
        );

        // All nodes have at most one flowing edge
        for node in branch.graph.node_indices() {
            let flowing = branch
                .graph
                .edges_directed(node, Outgoing)
                .filter(|e| e.weight().has_flow)
                .count();
            assert!(flowing <= 1);
        }

        // All nodes with two edges (one not flowing) has one that's flowing
        for node in branch.graph.node_indices() {
            if branch
                .graph
                .edges_directed(node, Outgoing)
                .any(|e| !e.weight().has_flow)
            {
                assert!(
                    branch
                        .graph
                        .edges_directed(node, Outgoing)
                        .any(|e| e.weight().has_flow)
                );
            }
        }

        // All non-source non-sink nodes have flow-in = flow-out + flow-absorbed
        for (index, room) in branch.graph.node_references() {
            if index == branch.source.0 || index == branch.sink {
                continue;
            }

            let flow_in = branch
                .graph
                .edges_directed(index, Incoming)
                .map(|e| e.weight().flow_seen)
                .sum::<u8>();

            let flow_out = room.flow_absorbed
                + branch
                    .graph
                    .edges_directed(index, Outgoing)
                    .map(|e| e.weight().flow_seen)
                    .sum::<u8>();

            assert_eq!(flow_in, flow_out);
        }
    }

    #[test]
    fn test_new() {
        let branch = Branch::new();
        assert_valid(&branch);
        assert_eq!(branch.graph.edge_count(), 0);
    }

    #[test]
    fn test_extend() {
        let mut branch = Branch::new();
        branch.extend_source();
        assert_valid(&branch);
        assert_eq!(branch.graph.edge_count(), 1);
    }

    #[test]
    fn test_extend_join_and_fork() {
        let mut branch = Branch::new();
        branch.extend_join_and_fork(NonZero::new(5).unwrap(), NonZero::new(5).unwrap());
        assert_valid(&branch);
        assert_eq!(branch.graph.edge_count(), 5 + 5 + 1);
    }
}
