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
    fn on_level_init(level: InitLevel) {
        if matches!(level, InitLevel::Scene) {
            godot_print!("Hello from Lizard State Godot Glue!");
        }
    }
}
