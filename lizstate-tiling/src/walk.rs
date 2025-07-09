//! A sequence of edges, described by directions.
//! Without knowing the graph we are walking on, we cannot know if a walk is a path (no repeated vertices)
//! We can *sort of* know if a walk is a trail (no repeated edges).

#[derive(Debug)]
pub struct WalkIsEmpty;

#[derive(Debug)]
pub struct WalkIsFull;

// Yeah
pub mod traits;

// Implementations
pub mod enumeration;
#[cfg(feature = "std")]
pub mod vec;

// Wrappers
pub mod reduced;
