//! 2D spaces (plural!) formed by joining square tiles.
//!
//! # Usage
//! All Copy + Eq traits are intended to be implemented on references or unit types.
//! All Clone + Eq traits are intended to be implemented on references, unit types, or smart pointers.
//!
//! # Contributing
//! Careful about making marker traits `Default.` Reference types cannot be `Default.`

#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![no_std]
#[cfg(feature = "std")]
extern crate std;

// *************** Core ***************

/// The four cardinal directions!
pub mod direction;

/// A pair of integers!
pub mod coords;

/// Sequences of directions.
pub mod walk;

/// Traits for 4-regular undirected graphs and their vertices.
///
/// Outgoing directed edges are named.
pub mod tiling_graph;

// *************** Example Graphs ***************

/// The familiar 2D coordinates.
pub mod euclidean_plane;

/// A big ass graph.
pub mod free_group;

// *************** Customizable Graphs ***************

/// A space defined by the user, following specific rules.
#[cfg(feature = "std")]
pub mod custom_space;

/// Replaces every tile of a space with many tiles.
pub mod expansion;
