#![warn(clippy::pedantic)]
// Clippy wants to pass `Gd<_>` by reference.
// However, this would need a lifetime, and the bindings don't work with generics.
#![allow(clippy::needless_pass_by_value)]
// Functions will mostly be called from Godot, and must_use wouldn't enforce anything there.
// I suppose it's fine though.
// #![allow(clippy::must_use_candidate)]

mod actions;
mod events;
mod floor;
mod positional;

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
