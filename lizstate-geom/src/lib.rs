//! A 2D space, divided into square tiles.

#![allow(dead_code, reason = "mid move")]
#![warn(clippy::pedantic)]
#![warn(clippy::allow_attributes_without_reason)]
#![allow(clippy::missing_errors_doc, reason = "TODO: Working on documentation")]

/// Integer math on a grid plane.
/// The old module we want to deprecate.
/// We can impl Into and From to do that.
pub mod positional;

/// Things you can do with `AbsolutePositions` and `RelativePositions`.
/// No effort is made to make this generic.
pub mod algorithms;
pub mod fov;
pub mod pathfinding;

// We want to keep this one, but it uses the old classes.
pub mod shapes;

// ------- PLANNED REFACTOR ----------

/// A trait for square grid movement, helpers.
///
/// Can be thought of as a 4-connected graph.
/// This library will refer to the nodes of the graph as `Tiles`.
/// The outgoing edges of each `Tile` have a `Direction`, with rules for how they behave.
pub mod grid;

/// A boring implementation of `Grid`.
///
/// It's a plane!
pub mod coords;

/// A trait for induced subgraphs, and an implementer, `IndoorMap`.
///
/// Useful for making "indoor maps."
pub mod induced_subgraph;

/// The least structured implementation of `Grid`.
///
/// Notably, there are multiple elements with the same projected coordinate, hence the `PlanarProjection` trait.
pub mod free_group;

/// A trait for adding edges while maintaining planarity, and an implementer.
///
/// Useful for connecting disconnected parts.
pub mod edge_supergraph;

/// Replaces each element of the grid with a bounded grid.
pub mod chunks;

/// The faces of the grid subgraph.
pub mod dual;

/// Flips and Rotations.
pub mod isometry;

/// The float extension of tiles.
pub mod point;

/// Points, lines, shapes on the plane.
///
/// Does not make global assumptions!
/// Maps can be provided to restrict how geometry works.
/// Layered maps too.
pub mod geometry;
