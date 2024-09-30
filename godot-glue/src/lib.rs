#![warn(clippy::pedantic)]
// Personal taste.
#![allow(clippy::module_name_repetitions)]
// Clippy wants to pass `Gd<_>` by reference.
// However, this would need a lifetime, and the bindings don't work with generics.
#![allow(clippy::needless_pass_by_value)]
// Functions will mostly be called from Godot, and must_use wouldn't enforce anything there.
// I suppose it's fine though.
// #![allow(clippy::must_use_candidate)]

pub mod actions;
pub mod events;
pub mod floor;
pub mod positional;
/// One way conversions from Godot to Rust.
/// Not intended for saving an existing game.
pub mod resources;

use std::default::Default;

use godot::prelude::*;
use tracing_subscriber::layer::SubscriberExt;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        if matches!(level, InitLevel::Scene) {
            // TODO: Find a better entrypoint(?), since this also needlessly runs on the editor.

            println!("subscribing to tracy!");
            let ignore_result = tracing::subscriber::set_global_default(
                tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
            );
            if ignore_result.is_err() {
                println!("already subscribed!");
            }
        }
    }
}
