//! Newtypes and their associated functions, understood by Godot.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions, reason = "Personal preference")]
#![allow(
    clippy::needless_pass_by_value,
    reason = "Functions exposed to Godot need params understood by Godot. References with lifetimes are not supported."
)]

pub mod actions;
pub(crate) mod events;
pub mod floor;
pub(crate) mod positional;
/// One way conversions from Godot to Rust.
/// Not intended for saving an existing game.
pub(crate) mod resources;

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
