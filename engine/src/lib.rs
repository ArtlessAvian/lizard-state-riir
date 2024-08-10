//! A traditional roguelike engine.
//!
//! The main user is me (artlessAvian) for my game Lizard State.
//! I suppose it's possible to make a similar game, make a different frontend, or mod.
//! (license with me first if you're fr, ig, idk. im permissive, probably.)
//!
//! Some mechanics this engine is designed around:
//! * Stateful entities (between turns)
//! * Queueing actions in the future
//!     * Interruping queued actions
//! * Fine grained positions
//!     * Multi-tile entities
//!     * Weird grid math
//!
//! The *game* design for those decisions is out of scope here. TL;DR it's sort of fighting game like.
//!
//! This is a reimplementation of the C# game at <https://github.com/artlessavian/lizard-state>.
//!
//! TODO:
//! * Add a module for the outer/meta game (decide if this goes in Godot or not).
//!     * Sequence of `Floor`s
//!     * Items, their associated actions.
//!     * Story/file progress.
//!     * Recurring characters.

#![warn(clippy::pedantic)]
// Personal taste.
#![allow(clippy::module_name_repetitions)]
// TODO: This is genuinely a problem, but we probably want to move to propogating Result eventually.
#![allow(clippy::missing_panics_doc)]
#![warn(clippy::clone_on_ref_ptr)]

/// Defining action related traits, and implementations of that trait.
pub mod actions;

/// The logic and data of the main game. The map, turntaking, etc.
pub mod floor;

/// A bundle of data, usually in the a context of a floor.
pub mod entity;

pub mod strategy;

/// Integer math and algorithms on a grid.
///
/// TODO: Maybe rename to something grid related?
pub mod positional;

/// Functional Magic.
mod writer;
