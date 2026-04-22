//! Godot friendly interaction with the Lizard State's `Dungeon` type in Rust.
//!
//! Game developers familiar with Godot should feel comfortable constructing a Dungeon and interacting with it.
//! This crate uses the same mental model.
//!
//! High-level users can interact with this crate's nodes through the scripts and the editor.
//! They can save scenes in the editor, or even at runtime.
//! Scenes contain the data needed to construct a `Dungeon.`
//!
//! Low-level users can interact with this crate's server through scripts.
//! Data in servers are *not* saved in the scene, unless stored in a node.
//! The server can return event data or fire signals.
//!
//! # Scope
//! This is only responsible for the `Dungeon.`
//! Other features of the Lizard State can be written as scripts, where Rust's strictness is not as necessary.

#![warn(clippy::pedantic)]
#![warn(clippy::allow_attributes_without_reason)]
#![allow(
    clippy::needless_pass_by_value,
    reason = "Functions exposed to Godot need params understood by Godot. References with lifetimes are not supported."
)]

use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_stage_init(level: InitStage) {
        if matches!(level, InitStage::Scene) {
            godot_print!("Hello from Lizard State GDExtension!");
        }
    }
}
