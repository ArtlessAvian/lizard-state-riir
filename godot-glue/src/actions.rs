pub(crate) mod public;

use std::borrow::Cow;
use std::rc::Rc;

use engine::actions::ActionTrait;
use engine::actions::BoxedCommand;
use engine::actions::DirectionActionTrait;
use engine::actions::InfallibleActionTrait;
use engine::actions::TileActionTrait;
use godot::prelude::*;

use crate::floor::ActiveFloor;
use crate::floor::EntityId;

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
    #[must_use]
    pub fn to_command(&self, floor: Gd<ActiveFloor>, subject: Gd<EntityId>) -> Option<Gd<Command>> {
        let binding = subject.bind();
        let subject_id = binding.id;
        let verify_action = self
            .action
            .verify_and_box(&Cow::Owned(floor.bind().internal.clone()), subject_id)
            .ok()?;
        Some(Command::new(verify_action))
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
    #[must_use]
    pub fn to_command(
        &self,
        floor: Gd<ActiveFloor>,
        subject: Gd<EntityId>,
        tile: crate::positional::AbsolutePosition,
    ) -> Option<Gd<Command>> {
        let binding = subject.bind();
        let subject_id = binding.id;
        let verify_action = self
            .action
            .verify_and_box(
                &Cow::Owned(floor.bind().internal.clone()),
                subject_id,
                tile.into(),
            )
            .ok()?;
        Some(Command::new(verify_action))
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
    #[must_use]
    pub fn to_command(
        &self,
        floor: Gd<ActiveFloor>,
        subject: Gd<EntityId>,
        dir: crate::positional::RelativePosition,
    ) -> Option<Gd<Command>> {
        let binding = subject.bind();
        let subject_id = binding.id;
        let verify_action = self
            .action
            .verify_and_box(
                &Cow::Owned(floor.bind().internal.clone()),
                subject_id,
                dir.into(),
            )
            .ok()?;
        Some(Command::new(verify_action))
    }
}

/// An opaque object containing an Action. Has no logic.
#[derive(GodotClass)]
#[class(no_init)]
pub struct InfallibleAction {
    // Godot doesn't see this anyways.
    action: Rc<dyn InfallibleActionTrait>,
}

#[godot_api]
impl InfallibleAction {
    pub fn new(action: Rc<dyn InfallibleActionTrait>) -> Gd<Self> {
        Gd::from_object(Self { action })
    }

    #[func]
    #[must_use]
    pub fn to_command(&self, floor: Gd<ActiveFloor>, subject: Gd<EntityId>) -> Gd<Command> {
        let binding = subject.bind();
        let subject_id = binding.id;
        let verify_action = self
            .action
            .verify_and_box(&Cow::Owned(floor.bind().internal.clone()), subject_id);
        Command::new(verify_action)
    }
}

/// An opaque object representing a Command. Has no logic.
///
/// Note the inversion between object and param compared to Engine (though that may change).
/// ```rust
/// use godot::prelude::*;
/// use godot_glue::actions::Command;
/// use godot_glue::floor::ActiveFloor;
/// use engine::actions::BoxedCommand;
/// use engine::floor::Floor;
/// fn engine_context(floor: &Floor, command: BoxedCommand) {
///     command.do_action(&floor);
/// }
/// fn glue_context(floor: &mut ActiveFloor, command: Gd<Command>) {
///     floor.do_action(command);
/// }
/// ```
#[derive(GodotClass)]
#[class(no_init)]
pub struct Command {
    // Godot doesn't see this anyways.
    pub command: Option<BoxedCommand<'static>>,
}

#[godot_api]
impl Command {
    #[must_use]
    pub fn new(command: BoxedCommand<'static>) -> Gd<Self> {
        Gd::from_object(Self {
            command: Some(command),
        })
    }
}
