use godot::prelude::*;

#[derive(GodotConvert, Var, Debug)]
#[godot(transparent)]
pub struct RelativePosition {
    relative: Vector2i,
}

impl From<engine::positional::RelativePosition> for RelativePosition {
    fn from(value: engine::positional::RelativePosition) -> Self {
        Self {
            relative: Vector2i {
                x: value.dx,
                y: value.dy,
            },
        }
    }
}

impl From<RelativePosition> for engine::positional::RelativePosition {
    fn from(value: RelativePosition) -> Self {
        Self {
            dx: value.relative.x,
            dy: value.relative.y,
        }
    }
}

#[derive(GodotConvert, Var, Debug)]
#[godot(transparent)]
pub struct AbsolutePosition {
    absolute: Vector2i,
}

impl From<engine::positional::AbsolutePosition> for AbsolutePosition {
    fn from(value: engine::positional::AbsolutePosition) -> Self {
        Self {
            absolute: Vector2i {
                x: value.x,
                y: value.y,
            },
        }
    }
}

impl From<AbsolutePosition> for engine::positional::AbsolutePosition {
    fn from(value: AbsolutePosition) -> Self {
        Self {
            x: value.absolute.x,
            y: value.absolute.y,
        }
    }
}
