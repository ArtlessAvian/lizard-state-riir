//! Construction and representation of a `Dungeon` in Lizard State.
//!
//! The crate is confined to this scope *because* it is written in Rust.
//! Rust is used for maintaining invariants, good error handling, and sum types (enums).
//!
//! The crate is tied heavily *to* the Lizard State as a concrete game.
//! We do not care about repurposing this crate for other games.
//! This lets us focus on our own game design.
//! We will never need `dyn Trait`.

// To developers: (aka reminder to self)
// * Be careful with `pub`!
// * You aren't going to need it.
// * Every addition should be motivated from the Godot Extension crate.

#![warn(clippy::pedantic)]
#![warn(clippy::allow_attributes_without_reason)]
#![warn(clippy::unwrap_used)]

/// `Cardinal`s and `Path`s.
pub mod spatial;
