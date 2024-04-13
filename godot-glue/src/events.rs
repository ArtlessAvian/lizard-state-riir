use engine::positional::AbsolutePosition;
use godot::prelude::*;

use crate::EntityId;

#[derive(GodotClass)]
#[class(no_init)]
pub struct MoveEvent {
    #[export]
    pub subject: Gd<EntityId>,
    // #[export]
    pub tile: AbsolutePosition,
}

// TODO: think about how these will be constructed from the engine enum.
// something like
// match event {
//  case MyEvent(e): {MyEvent::new(e)}
// }
// doesn't seem too bad.

// TODO: also export relativeposition and such as vector2i and other godot primitives.
