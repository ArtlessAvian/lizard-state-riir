//! A 2D space, divided into square tiles.

#![allow(dead_code, reason = "mid move")]
#![warn(clippy::pedantic)]
#![warn(clippy::allow_attributes_without_reason)]
#![allow(clippy::missing_errors_doc, reason = "TODO: Working on documentation")]

/// Integer math and algorithms on a grid.
///
/// TODO: Maybe rename to something grid related?
pub mod positional;

pub mod algorithms;
pub mod fov;
pub mod pathfinding;
pub mod shapes;
