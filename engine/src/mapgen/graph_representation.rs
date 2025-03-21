#![allow(dead_code, reason = "WIP")]

pub mod generator;

use std::num::NonZero;
use std::ops::Deref;

use petgraph::Graph;
use petgraph::acyclic::Acyclic;
use petgraph::acyclic::AcyclicEdgeError;
use petgraph::data::Build;
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
pub enum DegreeLimitError {
    Incoming,
    Outgoing,
    Total,
}

#[derive(Debug)]
pub struct InDegreeZero(NodeIndex<u8>);
impl From<InDegreeZero> for NodeIndex<u8> {
    fn from(value: InDegreeZero) -> Self {
        value.0
    }
}
impl Deref for InDegreeZero {
    type Target = NodeIndex<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ValidParent(NodeIndex<u8>);
impl ValidParent {
    fn try_from_with_proof<A, B>(
        index: NodeIndex<u8>,
        graph: &Graph<A, B, Directed, u8>,
    ) -> Result<Self, DegreeLimitError> {
        if graph.edges(index).count() >= 3 {
            Err(DegreeLimitError::Total)
        } else if graph.edges_directed(index, Incoming).count() >= 2 {
            Err(DegreeLimitError::Incoming)
        } else {
            Ok(Self(index))
        }
    }
}
impl From<ValidParent> for NodeIndex<u8> {
    fn from(value: ValidParent) -> Self {
        value.0
    }
}
impl Deref for ValidParent {
    type Target = NodeIndex<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Hopefully me when I tell my parents about my game dev career.
#[derive(Debug)]
pub struct ValidChild(NodeIndex<u8>);
impl ValidChild {
    fn try_from_with_proof<A, B>(
        index: NodeIndex<u8>,
        graph: &Graph<A, B, Directed, u8>,
    ) -> Result<Self, DegreeLimitError> {
        if graph.edges(index).count() >= 3 {
            Err(DegreeLimitError::Total)
        } else if graph.edges_directed(index, Outgoing).count() >= 2 {
            Err(DegreeLimitError::Outgoing)
        } else {
            Ok(Self(index))
        }
    }
}
impl From<InDegreeZero> for ValidChild {
    fn from(value: InDegreeZero) -> Self {
        Self(value.0)
    }
}
impl From<ValidChild> for NodeIndex<u8> {
    fn from(value: ValidChild) -> Self {
        value.0
    }
}
impl Deref for ValidChild {
    type Target = NodeIndex<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
    pub graph: Acyclic<Graph<Room, Hall, Directed, u8>>,
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
        let mut graph: Acyclic<Graph<Room, Hall, Directed, u8>> = Acyclic::new();
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
        self.add_new_parent(ValidChild(self.source.0));
    }

    // Don't pass in two ones, or itll be a multigraph.
    pub fn extend_join_and_fork(&mut self, wet_edges: NonZero<u8>, dry_edges: NonZero<u8>) {
        let shared_join = self.source.0;
        for _ in 0..(u8::from(wet_edges)) {
            self.extend_source();
        }

        let mut dry_destination = ValidChild::try_from_with_proof(shared_join, &self.graph)
            .expect("The old source should now have 1 in and 1 out");
        for _ in 0..(u8::from(dry_edges) - 1) {
            dry_destination = self.add_new_parent(dry_destination).into();
        }

        let shared_fork = self.source.0;

        // Source pointing into the Fork.
        self.extend_source();

        let shared_fork = ValidParent::try_from_with_proof(shared_fork, &self.graph)
            .expect("The old source should have 1 in and 1 out");

        self.add_edge(shared_fork, dry_destination, Hall::new(false))
            .expect("We know shared_fork's parent (and only ancestor) has 0 in.");
    }

    pub fn add_new_child(&mut self, parent: ValidParent) -> ValidParent {
        if self.sink == *parent {
            let new_sink = self.graph.add_node(Room {
                has_flow: true,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph
                .try_add_edge(parent.into(), new_sink, Hall::new(true))
                .expect("New edge to new node cannot be a cycle");

            self.sink = new_sink;
            ValidParent(new_sink)
        } else {
            let new_child = self.graph.add_node(Room {
                has_flow: false,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph
                .try_add_edge(parent.into(), new_child, Hall::new(false))
                .expect("New edge to new node cannot be a cycle");

            ValidParent(new_child)
        }
    }

    pub fn add_new_parent(&mut self, child: ValidChild) -> InDegreeZero {
        if *child == self.source.0 {
            let new_source = self.graph.add_node(Room {
                has_flow: true,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph
                .try_add_edge(new_source, child.into(), Hall::new(true))
                .expect("New edge from new node cannot be a cycle");

            self.source.0 = new_source;
            InDegreeZero(new_source)
        } else {
            let new_parent = self.graph.add_node(Room {
                has_flow: false,
                flow_seen: 0,
                flow_absorbed: 0,
            });
            self.graph
                .try_add_edge(new_parent, child.into(), Hall::new(true))
                .expect("New edge from new node cannot be a cycle");

            InDegreeZero(new_parent)
        }
    }

    // We can be sure the degree limits are held,
    // but we cannot protect against the error case here.
    // External logic must be used.
    pub fn add_edge(
        &mut self,
        from: ValidParent,
        to: ValidChild,
        weight: Hall,
    ) -> Result<EdgeIndex<u8>, AcyclicEdgeError<NodeIndex<u8>>> {
        self.graph.try_add_edge(from.into(), to.into(), weight)
    }

    pub fn get_candidate_parents(&self) -> impl Iterator<Item = ValidParent> {
        self.graph
            .node_indices()
            .filter(|a| *a != self.source.0)
            .filter_map(|a| ValidParent::try_from_with_proof(a, &self.graph).ok())
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
        // Assertion broken by dead ends, with no flow in, and two children with no flow.

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

    #[test]
    fn test_add_children() {
        let mut branch = Branch::new();
        branch.extend_source();

        for lol in branch.get_candidate_parents().collect::<Box<_>>() {
            branch.add_new_child(lol);
        }

        assert_valid(&branch);
    }

    #[test]
    fn test_monstrosity() {
        let mut branch = Branch::new();
        for _ in 0..5 {
            branch.extend_join_and_fork(NonZero::new(3).unwrap(), NonZero::new(3).unwrap());
            for lol in branch.get_candidate_parents().collect::<Box<_>>() {
                branch.add_new_child(lol);
            }
        }
        assert_valid(&branch);
    }
}
