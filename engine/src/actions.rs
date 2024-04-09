mod example;

/// Publicly exposed actions, free to construct.
pub mod public;

use std::rc::Rc;

use crate::data::Floor;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;
/// An action, without definining a user or a context.
///
/// A generic flow.
/// TODO: Rethink this.
/// ```rust
/// use engine::actions::*;
/// use engine::data::*;
/// fn context(action: Box<dyn ActionTrait>, floor: &Floor) -> (Floor, Vec<FloorEvent>) {
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
    fn do_action(&self, floor: &Floor) -> (Floor, Vec<FloorEvent>);
}

/// Never verifies.
pub struct NullAction {}
impl ActionTrait for NullAction {
    fn verify_action(&self, _: &Floor, _: &Rc<Entity>) -> Option<Box<dyn CommandTrait>> {
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FloorEvent {
    Move(EntityId, AbsolutePosition),
    StartAttack(EntityId, RelativePosition),
    AttackHit(EntityId, EntityId, i32),
}
