mod example;

// TOOD: move to data, which should also be renamed floor/gamestate.
pub mod events;

/// Publicly exposed actions, free to construct.
pub mod public;

use std::rc::Rc;

use crate::data::Floor;
use crate::entity::Entity;

use self::events::FloorEvent;

/// An action, something that someone could do. Who and when is not defined.
///
/// Sort of like a dry run. If this returned floors immediately, it would be "most correct"
/// but also create a lot of allocations.
///
/// A generic flow.
/// ```rust
/// use std::rc::Rc;
/// use engine::actions::*;
/// use engine::data::*;
/// use engine::entity::*;
/// fn context(action: Box<dyn ActionTrait>, floor: &Floor, player_id: Rc<Entity>) -> (Floor, Vec<FloorEvent>) {
///     let command = action.verify_action(floor, &player_id).unwrap();
///     command.do_action(floor)
/// }
/// ```
///
/// TODOs
/// * Maybe create another trait to describe an UnaimedAction? Currently you construct an action with the aiming built in.
///   This would involve yet more type erasure. You would know what UnaimedAction you have, but you would forget when you aim it.
/// * Consider if subject should **always** the floor's next turn taker.
/// * If not, can actions/commands call each other?

pub trait ActionTrait {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
    ) -> Option<Box<dyn CommandTrait>>;
}

/// Someone, doing something, in some context. Can panic!
///
/// If there is an error, it is not the user's fault. So panicking is fine.
///
/// Unfortunately the context is not stored, so you have to pass it back in again. Which leads to:
/// TODO: Figure out lifetime shenanigans, so Commands live shorter than Actions.
/// You shouldn't need to store a command, you just do it.
/// If you must, you store the action that produced it.

pub trait CommandTrait {
    fn do_action(&self, floor: &Floor) -> (Floor, Vec<FloorEvent>);
}

/// An action that never verifies to a command.
///
/// This is preferable to a no-op command, since that would produce a new Floor.
pub struct NullAction {}
impl ActionTrait for NullAction {
    fn verify_action(&self, _: &Floor, _: &Rc<Entity>) -> Option<Box<dyn CommandTrait>> {
        None
    }
}
