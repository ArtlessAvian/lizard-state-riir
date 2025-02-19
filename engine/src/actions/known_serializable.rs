use std::fmt::Debug;
use std::rc::Rc;

use enum_dispatch::enum_dispatch;
use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::characters::axolotl_nano::EnterSmiteStanceAction;
use super::characters::axolotl_nano::StanceSmiteAction;
use super::characters::max_tegu::ForwardHeavyAction;
use super::characters::max_tegu::ForwardHeavyFollowupAction;
use super::characters::max_tegu::TrackingAction;
use super::characters::max_tegu::TrackingFollowupAction;
use super::example::DoubleHitAction;
use super::example::DoubleHitFollowupAction;
use super::example::EnterStanceAction;
use super::example::ExitStanceAction;
use super::public::GoingAction;
use super::ActionError;
use super::BoxedCommand;
use super::InfallibleActionTrait;
use super::Never;
use super::SerializeActionTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeInfallibleActionTrait;
use super::SerializeTileActionTrait;
use super::UnaimedAction;
use super::UnaimedMacroTrait;
use super::UnaimedTrait;
use crate::entity::EntityId;
use crate::floor::Floor;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

// SerializableAction -- Enum -> SerializableUnaimedAction -- Erasure -> UnaimedAction
//      ^ Enum ^                        ^ Into ^
//    KnownAction     -- Enum ->    KnownUnaimedAction

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum SerializableUnaimedAction {
    None(SerializableAction),
    Tile(SerializableTileAction),
    Direction(SerializableDirectionAction),
    Infallible(SerializableInfallibleAction),
}

impl From<SerializableUnaimedAction> for UnaimedAction {
    fn from(value: SerializableUnaimedAction) -> Self {
        // mmm yes i love indirection
        match value {
            SerializableUnaimedAction::None(x) => UnaimedAction::None(Rc::new(x)),
            SerializableUnaimedAction::Tile(x) => UnaimedAction::Tile(Rc::new(x)),
            SerializableUnaimedAction::Direction(x) => UnaimedAction::Direction(Rc::new(x)),
            SerializableUnaimedAction::Infallible(x) => UnaimedAction::Infallible(Rc::new(x)),
        }
    }
}

/// `UnaimedActions` known to this crate!
///
/// Other crates should create a similar type, and use the Erased Field in the `SerializableAction` types.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum KnownUnaimedAction {
    // None(KnownAction),
    Tile(KnownTileAction),
    Direction(KnownDirectionAction),
    Infallible(KnownInfallibleAction),
}

impl From<KnownUnaimedAction> for SerializableUnaimedAction {
    fn from(value: KnownUnaimedAction) -> Self {
        match value {
            KnownUnaimedAction::Tile(x) => Self::Tile(SerializableTileAction::Known(x)),
            KnownUnaimedAction::Direction(x) => {
                Self::Direction(SerializableDirectionAction::Known(x))
            }
            KnownUnaimedAction::Infallible(x) => {
                Self::Infallible(SerializableInfallibleAction::Known(x))
            }
        }
    }
}

#[derive(Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(ActionTrait)]
pub enum SerializableAction {
    // Known(KnownAction),
    Erased(Rc<dyn SerializeActionTrait>),
}

impl Debug for SerializableAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Self::Known(arg0) => f.debug_tuple("Known").field(arg0).finish(),
            Self::Erased(_) => f.debug_tuple("External").finish_non_exhaustive(),
        }
    }
}

impl UnaimedTrait for Rc<dyn SerializeActionTrait> {
    type Target = ();
    type Error = ActionError;
}

impl UnaimedMacroTrait for Rc<dyn SerializeActionTrait> {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        (): (),
    ) -> Result<BoxedCommand, ActionError> {
        self.as_ref().verify_and_box(floor, subject_id)
    }
}

// #[derive(Debug, Clone, Archive, Serialize, Deserialize)]
// pub enum KnownAction {}

#[derive(Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(TileActionTrait)]
pub enum SerializableTileAction {
    Known(KnownTileAction),
    Erased(Rc<dyn SerializeTileActionTrait>),
}

impl Debug for SerializableTileAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Known(arg0) => f.debug_tuple("Known").field(arg0).finish(),
            Self::Erased(_) => f.debug_tuple("External").finish_non_exhaustive(),
        }
    }
}

impl UnaimedTrait for Rc<dyn SerializeTileActionTrait> {
    type Target = AbsolutePosition;
    type Error = ActionError;
}

impl UnaimedMacroTrait for Rc<dyn SerializeTileActionTrait> {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Result<BoxedCommand, ActionError> {
        self.as_ref().verify_and_box(floor, subject_id, tile)
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(TileActionTrait)]
pub enum KnownTileAction {
    Tracking(TrackingAction),
    EnterSmiteStance(EnterSmiteStanceAction),
}

#[derive(Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(DirectionActionTrait)]
pub enum SerializableDirectionAction {
    Known(KnownDirectionAction),
    Erased(Rc<dyn SerializeDirectionActionTrait>),
}

impl Debug for SerializableDirectionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Known(arg0) => f.debug_tuple("Known").field(arg0).finish(),
            Self::Erased(_) => f.debug_tuple("External").finish_non_exhaustive(),
        }
    }
}

impl UnaimedTrait for Rc<dyn SerializeDirectionActionTrait> {
    type Target = RelativePosition;
    type Error = ActionError;
}

impl UnaimedMacroTrait for Rc<dyn SerializeDirectionActionTrait> {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Result<BoxedCommand, ActionError> {
        self.as_ref().verify_and_box(floor, subject_id, dir)
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(DirectionActionTrait)]
pub enum KnownDirectionAction {
    DoubleHit(DoubleHitAction),
    ForwardHeavy(ForwardHeavyAction),
}

#[derive(Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(InfallibleActionTrait)]
pub enum SerializableInfallibleAction {
    Known(KnownInfallibleAction),
    Erased(Rc<dyn SerializeInfallibleActionTrait>),
}

impl Debug for SerializableInfallibleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Known(arg0) => f.debug_tuple("Known").field(arg0).finish(),
            Self::Erased(_) => f.debug_tuple("Erased").finish_non_exhaustive(),
        }
    }
}

impl UnaimedTrait for Rc<dyn SerializeInfallibleActionTrait> {
    type Target = ();
    type Error = Never;
}

impl UnaimedMacroTrait for Rc<dyn SerializeInfallibleActionTrait> {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        (): (),
    ) -> Result<BoxedCommand, Never> {
        Ok(InfallibleActionTrait::verify_and_box(
            self.as_ref(),
            floor,
            subject_id,
        ))
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(InfallibleActionTrait)]
pub enum KnownInfallibleAction {
    EnterStance(EnterStanceAction),
    ExitStance(ExitStanceAction),
    StanceSmite(StanceSmiteAction),
    DoubleHitFollowupAction(DoubleHitFollowupAction),
    ForwardHeavyFollowupAction(ForwardHeavyFollowupAction),
    TrackingFollowupAction(TrackingFollowupAction),
    GoingAction(GoingAction),
}
