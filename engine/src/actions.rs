mod example;

/// Publicly exposed actions, free to construct.
pub mod public;

use std::rc::Rc;

use crate::data::Entity;
use crate::data::Floor;

/// An action, without definining a user or a context.
///
/// A generic flow.
/// ```rust
/// use engine::actions::*;
/// use engine::data::*;
/// fn context(action: Box<dyn ActionTrait>, floor: &Floor) -> Floor {
///     let command = action.verify_action(floor, &floor.get_player()).unwrap();
///     command.do_action(floor)
/// }
/// ```
pub trait ActionTrait {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
    ) -> Option<Box<dyn CommandTrait>>;
}

/// A command, with user and context.
pub trait CommandTrait {
    fn do_action(&self, floor: &Floor) -> Floor;
}

/// Never verifies.
pub struct NullAction {}
impl ActionTrait for NullAction {
    fn verify_action(&self, _: &Floor, _: &Rc<Entity>) -> Option<Box<dyn CommandTrait>> {
        None
    }
}
