use std::rc::Rc;

use engine::actions::ActionTrait;
use engine::actions::CommandTrait;
use engine::actions::DirectionActionTrait;
use engine::actions::TileActionTrait;
use godot::prelude::*;

use crate::floor::EntityId;
use crate::floor::Floor;

/// An opaque object containing an Action. Has no logic.
#[derive(GodotClass)]
#[class(no_init)]
pub struct Action {
    // Godot doesn't see this anyways.
    action: Rc<dyn ActionTrait>,
}

#[godot_api]
impl Action {
    pub fn new(action: Rc<dyn ActionTrait>) -> Gd<Self> {
        Gd::from_object(Self { action })
    }

    #[func]
    pub fn to_command(&self, floor: Gd<Floor>, subject: Gd<EntityId>) -> Option<Gd<Command>> {
        let binding = subject.bind();
        let subject_id = binding.id;
        let verify_action = self.action.verify_action(&floor.bind().floor, subject_id)?;
        Some(Command::new(verify_action.into()))
    }
}

/// An opaque object containing an Action. Has no logic.
#[derive(GodotClass)]
#[class(no_init)]
pub struct TileAction {
    // Godot doesn't see this anyways.
    action: Rc<dyn TileActionTrait>,
}

#[godot_api]
impl TileAction {
    pub fn new(action: Rc<dyn TileActionTrait>) -> Gd<Self> {
        Gd::from_object(Self { action })
    }

    #[func]
    pub fn to_command(
        &self,
        floor: Gd<Floor>,
        subject: Gd<EntityId>,
        tile: crate::positional::AbsolutePosition,
    ) -> Option<Gd<Command>> {
        let binding = subject.bind();
        let subject_id = binding.id;
        let verify_action =
            self.action
                .verify_action(&floor.bind().floor, subject_id, tile.into())?;
        Some(Command::new(verify_action.into()))
    }
}

/// An opaque object containing an Action. Has no logic.
#[derive(GodotClass)]
#[class(no_init)]
pub struct DirectionAction {
    // Godot doesn't see this anyways.
    action: Rc<dyn DirectionActionTrait>,
}

#[godot_api]
impl DirectionAction {
    pub fn new(action: Rc<dyn DirectionActionTrait>) -> Gd<Self> {
        Gd::from_object(Self { action })
    }

    #[func]
    pub fn to_command(
        &self,
        floor: Gd<Floor>,
        subject: Gd<EntityId>,
        dir: crate::positional::RelativePosition,
    ) -> Option<Gd<Command>> {
        let binding = subject.bind();
        let subject_id = binding.id;
        let verify_action =
            self.action
                .verify_action(&floor.bind().floor, subject_id, dir.into())?;
        Some(Command::new(verify_action.into()))
    }
}

/// An opaque object representing a Command. Has no logic.
///
/// Note the inversion between object and param compared to Engine (though that may change).
/// ```rust
/// // Note this will not run as a doctest since godot_glue is a cdylib.
/// use engine::data::Floor as FloorInternal;
/// use engine::actions::Command as CommandInternal;
/// use godot_glue::Floor;
/// use godot_glue::Command;
///
/// fn engine_context(floor: &FloorInternal, command: &Box<dyn CommandTrait>) {
///     command.do_action(floor);
/// }
/// fn glue_context(floor: &mut Floor, command: &Command) {
///     floor.do_action(command);
/// }
/// ```
#[derive(GodotClass)]
#[class(no_init)]
pub struct Command {
    // Godot doesn't see this anyways.
    pub command: Rc<dyn CommandTrait>,
}

#[godot_api]
impl Command {
    pub fn new(command: Rc<dyn CommandTrait>) -> Gd<Self> {
        Gd::from_object(Self { command })
    }
}
