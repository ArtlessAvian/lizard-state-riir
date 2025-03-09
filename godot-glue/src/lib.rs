//! Newtypes and their associated functions, understood by Godot.

#![warn(clippy::pedantic)]
#![expect(
    clippy::module_name_repetitions,
    reason = "Godot/GDScript cannot see module names, so module name repetition may be necessary."
)]
#![expect(
    clippy::needless_pass_by_value,
    reason = "Functions exposed to Godot need params understood by Godot. References with lifetimes are not supported."
)]

/// Godot `Resource` types that can build Rust objects.
///
/// Not all Rust structs can be converted back into a Godot `Resource`.
/// Consequently, this is not intended for saving an existing game.
/// This lets the Godot construct a SUBSET of what the Rust engine can represent.
/// For a level editor, this is good enough. Add as needed.
pub mod builders;

/// Godot exposed operations on Rust engine's opaque types.
///
/// Only some of the surface area is exposed as needed.
pub mod logic;

/// Godot views and conversions of the Rust engine's `Copy` types.
///
/// These are usually function params and such.
/// The values are simple and unlikely to change behavior.
/// We can do full back and forth conversions.
pub mod values;

#[cfg(feature = "profiling")]
use std::default::Default;

use godot::prelude::*;
#[cfg(feature = "profiling")]
use tracing_subscriber::layer::SubscriberExt;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        if matches!(level, InitLevel::Scene) {
            godot_print!("Hello from Lizard State Godot Glue!");

            #[cfg(feature = "profiling")]
            {
                // TODO: Find a better entrypoint(?), since this also needlessly runs on the editor.
                godot_print!("subscribing to tracy!");
                let ignore_result = tracing::subscriber::set_global_default(
                    tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
                );
                if ignore_result.is_err() {
                    godot_print!("already subscribed!");
                }
            }
        }
    }
}
