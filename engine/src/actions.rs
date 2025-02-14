#![allow(
    clippy::wildcard_imports,
    reason = "enum_dispatch macro needs the trait to know its implementors."
)]

pub(crate) mod utils;

pub mod example;

// TOOD: move to data, which should also be renamed floor/gamestate.
pub mod events;

/// Publicly exposed actions, free to construct.
pub mod public;

/// Actions to give to entities on creation. Access through `Entity::get_actions()`.
///
/// TODO: Move to dependent crate?
/// TODO: Ensure entity "owns" the action before creating command.
pub mod characters;

/// Enum of crate actions and commands also implementing rkyv traits.
///
/// The enums are "known" to the crate. This also avoids the overloaded "(de)serialize(able)" term.
///
/// # Serialization
/// Since each element of the enums also implement rkyv traits, then the enum itself can.
/// This is purely for ergonomics, avoiding boilerplate around the `rkyv_dyn` crate.
/// External crates are encouraged to write their own static dispatch enum and only implement the `rkyv_dyn` crate for that.
pub mod known_serializable;

use std::fmt::Debug;
use std::rc::Rc;

use enum_dispatch::enum_dispatch;
use rkyv_dyn::archive_dyn;

use self::characters::axolotl_nano::*;
use self::characters::max_tegu::*;
use self::events::FloorEvent;
use self::example::*;
use self::known_serializable::*;
use self::public::*;
use crate::entity::EntityId;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

#[derive(Debug)]
#[non_exhaustive]
pub enum ActionError {
    #[deprecated]
    Todo,
    TargetOutOfRange,
    DataMismatch,
    InvalidTarget,
    NotEnoughEnergy,
    FloorInvalid,
    MacroFallthrough,
    InvalidState,
}

// Rc to allow cloning trait objects, also its cheap!
#[derive(Debug, Clone)]
pub enum UnaimedAction {
    None(Rc<dyn ActionTrait>),
    Tile(Rc<dyn TileActionTrait>),
    Direction(Rc<dyn DirectionActionTrait>),
}

/// Shared thingy.
pub trait UnaimedTrait {
    type Target;
    type Error;
}

/// An action, something that someone could do. Who and when is not defined.
///
/// This does not immediately mutate the input floor or create a new Floor.
/// That is instead done by `CommandTrait`.
/// All user errors will be returned, though unrecoverable panic! may happen in `CommandTrait`.
///
/// Technically this is dyn compatible, but not very useful if so.
///
/// There are type erased traits `ActionTrait`, `TileActionTrait`, `DirectionActionTrait`, etc.
/// One may implement them without implementing this. If you do implement this, a blanket implementation
/// will be given. If you intend to have it saved, use `rkyv_dyn`'s `register_impl!(MyType as SomethingActionTrait)`.
///
/// Avoid setting the output type to `Wrapper<dyn CommandTrait>`, where `Wrapper<dyn CommandTrait>` implements `CommandTrait`.
/// This causes pointless indirection.
///
/// # Example Usage
/// ```rust
/// use engine::actions::*;
/// use engine::floor::*;
/// use engine::entity::*;
/// fn apply_action_to_floor<Action>(action: Action, floor: &Floor, player_id: EntityId, target: Action::Target) -> FloorUpdate
/// where
///     Action: UnaimedActionTrait
/// {
///     match action.verify(floor, player_id, target) {
///         Ok(command) => command.do_action(floor),
///         Err(err) => panic!(),
///     }
/// }
/// ```
pub trait UnaimedActionTrait: UnaimedTrait {
    type Command: CommandTrait + 'static;

    fn verify(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        target: Self::Target,
    ) -> Result<Self::Command, Self::Error>;

    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        target: Self::Target,
    ) -> Result<Box<dyn CommandTrait>, Self::Error> {
        match self.verify(floor, subject_id, target) {
            Ok(ok) => Ok(Box::new(ok)),
            Err(err) => Err(err),
        }
    }
}

/// `UnaimedActionTrait` with type erased Command. Dyn compatible!
///
/// If an action returns the result of two separate commands, this may come in handy!
/// You could write an enum and implement for that too, but its awkward.
///
/// See `UnaimedActionTrait`!
pub trait UnaimedMacroTrait: UnaimedTrait {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        target: Self::Target,
    ) -> Result<Box<dyn CommandTrait>, Self::Error>;
}

impl<T: UnaimedActionTrait> UnaimedMacroTrait for T {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        target: Self::Target,
    ) -> Result<Box<dyn CommandTrait>, Self::Error> {
        match self.verify(floor, subject_id, target) {
            Ok(ok) => Ok(Box::new(ok)),
            Err(err) => Err(err),
        }
    }
}

/// Dyn compatible version of `UnaimedMacroTrait`. Output command is type erased and Target type is known.
///
/// See `UnaimedActionTrait`!
#[archive_dyn(deserialize)]
#[enum_dispatch]
pub trait ActionTrait: Debug {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
    ) -> Result<Box<dyn CommandTrait>, ActionError>;
}

impl<T> ActionTrait for T
where
    T: UnaimedMacroTrait<Target = (), Error = ActionError> + Debug,
{
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
    ) -> Result<Box<dyn CommandTrait>, ActionError> {
        UnaimedMacroTrait::verify_and_box(self, floor, subject_id, ())
    }
}

/// Dyn compatible version of `UnaimedActionTrait`. Output command is type erased
///
/// See `UnaimedActionTrait`!
#[archive_dyn(deserialize)]
#[enum_dispatch]
pub trait TileActionTrait: Debug {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Result<Box<dyn CommandTrait>, ActionError>;
}

impl<T> TileActionTrait for T
where
    T: UnaimedMacroTrait<Target = AbsolutePosition, Error = ActionError> + Debug,
{
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Result<Box<dyn CommandTrait>, ActionError> {
        UnaimedMacroTrait::verify_and_box(self, floor, subject_id, tile)
    }
}

/// Dyn compatible version of `UnaimedActionTrait`. Output command is type erased
///
/// See `UnaimedActionTrait`!
#[archive_dyn(deserialize)]
#[enum_dispatch]
pub trait DirectionActionTrait: Debug {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Result<Box<dyn CommandTrait>, ActionError>;
}

impl<T> DirectionActionTrait for T
where
    T: UnaimedMacroTrait<Target = RelativePosition, Error = ActionError> + Debug,
{
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Result<Box<dyn CommandTrait>, ActionError> {
        UnaimedMacroTrait::verify_and_box(self, floor, subject_id, dir)
    }
}

/// Someone, doing something, in some context. Can panic!
///
/// If there is an error, it is not the user's fault. So panicking is fine.
///
/// Unfortunately the context is not stored, so you have to pass it back in again. Which leads to:
/// TODO: Figure out lifetime shenanigans, so Commands live shorter than Actions.
/// You shouldn't need to store a command, you just do it.
/// If you must, you store the action that produced it.

// Not *everything* that is a command needs to be a SerializeCommandTrait (generated by #[archive_dyn]).
// Don't do it if you don't need it, its more work and more boilerplate.
// (Also note that Actions don't return them. It is not likely that the user is verifying an action and immediately serializing it.)
// (A dyn SerializeCommandTrait is probably generated by other Commands.)
#[archive_dyn(deserialize)]
#[enum_dispatch]
pub trait CommandTrait: Debug {
    fn do_action(&self, floor: &Floor) -> FloorUpdate;
}
