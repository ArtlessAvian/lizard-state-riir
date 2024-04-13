use godot::prelude::*;

use crate::positional::AbsolutePosition;
use crate::EntityId;

#[derive(GodotClass)]
#[class(no_init)]
pub struct MoveEvent {
    #[var(get)]
    subject: Gd<EntityId>,
    #[var(get)]
    tile: AbsolutePosition,
}

// TODO: think about how these will be constructed from the engine enum.
// something like
// match event {
//  case MyEvent(e): {MyEvent::new(e)}
// }
// doesn't seem too bad.

// TODO: also export relativeposition and such as vector2i and other godot primitives.
