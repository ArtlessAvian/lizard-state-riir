use std::borrow::Cow;
use std::fmt::Debug;
use std::rc::Rc;

use super::ActionError;
use super::ActionTrait;
use super::BoxedCommand;
use super::DirectionActionTrait;
use super::InfallibleActionTrait;
use super::TileActionTrait;
use super::UnaimedAction;
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
use crate::entity::EntityId;
use crate::floor::Floor;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

#[derive(Debug, Clone)]
pub enum KnownUnaimedAction {
    None(KnownAction),
    Tile(KnownTileAction),
    Direction(KnownDirectionAction),
    Infallible(KnownInfallibleAction),
}

impl From<KnownUnaimedAction> for UnaimedAction {
    fn from(value: KnownUnaimedAction) -> Self {
        // mmm yes i love indirection
        match value {
            KnownUnaimedAction::None(x) => UnaimedAction::None(Rc::new(x)),
            KnownUnaimedAction::Tile(x) => UnaimedAction::Tile(Rc::new(x)),
            KnownUnaimedAction::Direction(x) => UnaimedAction::Direction(Rc::new(x)),
            KnownUnaimedAction::Infallible(x) => UnaimedAction::Infallible(Rc::new(x)),
        }
    }
}

#[derive(Debug, Clone)]
#[enum_delegate::implement(ActionTrait)]
pub enum KnownAction {
    Dingus(NoopAction),
}

/// This is temporary.
#[derive(Debug, Clone)]
pub struct NoopAction;
impl ActionTrait for NoopAction {
    fn verify_and_box<'a>(
        &self,
        _floor: &Cow<'a, Floor>,
        _subject_id: EntityId,
    ) -> Result<BoxedCommand<'a>, ActionError> {
        Err(ActionError::InvalidState)
    }
}

#[derive(Debug, Clone)]
#[enum_delegate::implement(TileActionTrait)]
pub enum KnownTileAction {
    Tracking(TrackingAction),
    EnterSmiteStance(EnterSmiteStanceAction),
}

#[derive(Debug, Clone)]
#[enum_delegate::implement(DirectionActionTrait)]
pub enum KnownDirectionAction {
    DoubleHit(DoubleHitAction),
    ForwardHeavy(ForwardHeavyAction),
}

#[derive(Debug, Clone)]
#[enum_delegate::implement(InfallibleActionTrait)]
pub enum KnownInfallibleAction {
    EnterStance(EnterStanceAction),
    ExitStance(ExitStanceAction),
    StanceSmite(StanceSmiteAction),
    DoubleHitFollowupAction(DoubleHitFollowupAction),
    ForwardHeavyFollowupAction(ForwardHeavyFollowupAction),
    TrackingFollowupAction(TrackingFollowupAction),
    GoingAction(GoingAction),
}
