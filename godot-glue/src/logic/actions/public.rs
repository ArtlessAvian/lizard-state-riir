use std::rc::Rc;

use engine::actions::public::BumpAction;
use engine::actions::public::GotoAction;
use engine::actions::public::StepAction;
use engine::actions::public::StepMacroAction;
use engine::actions::public::WaitAction;
use godot::prelude::*;

use crate::logic::actions::Action;
use crate::logic::actions::DirectionAction;
use crate::logic::actions::TileAction;

/// Mirrors the actions in `engine::actions::public`.
#[derive(GodotClass)]
#[class(init)]
struct PublicActions;

#[godot_api]
impl PublicActions {
    #[func]
    #[must_use]
    pub fn get_wait_action() -> Gd<Action> {
        Action::new(Rc::new(WaitAction))
    }

    #[func]
    #[must_use]
    pub fn get_step_action() -> Gd<DirectionAction> {
        DirectionAction::new(Rc::new(StepAction))
    }

    #[func]
    #[must_use]
    pub fn get_bump_action() -> Gd<DirectionAction> {
        DirectionAction::new(Rc::new(BumpAction))
    }

    #[func]
    #[must_use]
    pub fn get_step_macro_action() -> Gd<DirectionAction> {
        DirectionAction::new(Rc::new(StepMacroAction))
    }

    #[func]
    #[must_use]
    pub fn get_goto_action() -> Gd<TileAction> {
        TileAction::new(Rc::new(GotoAction))
    }
}
