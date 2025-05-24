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

/// Integer math on a grid plane.
///
/// Can be thought of as a 4-connected graph.
pub mod tiles;

/// A configurable subgraph of all possible `Tiles`.
pub mod map;

/// The faces of the grid graph.
pub mod dual;

/// A map with layers.
///
/// Adds *connections* to movement on the grid.
/// (Does not explain *HOW* the connections work.)
pub mod layered_map;

/// An example of a space with layers.
pub mod free_group;

/// A more practical use of the `free_group` module.
pub mod chunks;

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
