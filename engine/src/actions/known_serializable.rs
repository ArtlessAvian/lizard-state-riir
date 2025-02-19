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
use super::SerializeActionTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeInfallibleActionTrait;
use super::SerializeTileActionTrait;
use super::UnaimedAction;

// Same as above, but more specialized. Has an ugly conversion.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
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

#[derive(Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(ActionTrait)]
pub enum KnownAction {
    External(Rc<dyn SerializeActionTrait>),
}

impl Debug for KnownAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::External(_) => f.debug_tuple("External").finish_non_exhaustive(),
        }
    }
}

#[derive(Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(TileActionTrait)]
pub enum KnownTileAction {
    Tracking(TrackingAction),
    EnterSmiteStance(EnterSmiteStanceAction),
    External(Rc<dyn SerializeTileActionTrait>),
}

impl Debug for KnownTileAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tracking(arg0) => f.debug_tuple("Tracking").field(arg0).finish(),
            Self::EnterSmiteStance(arg0) => f.debug_tuple("EnterSmiteStance").field(arg0).finish(),
            Self::External(_) => f.debug_tuple("External").finish_non_exhaustive(),
        }
    }
}

#[derive(Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(DirectionActionTrait)]
pub enum KnownDirectionAction {
    DoubleHit(DoubleHitAction),
    ForwardHeavy(ForwardHeavyAction),
    External(Rc<dyn SerializeDirectionActionTrait>),
}

impl Debug for KnownDirectionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DoubleHit(arg0) => f.debug_tuple("DoubleHit").field(arg0).finish(),
            Self::ForwardHeavy(arg0) => f.debug_tuple("ForwardHeavy").field(arg0).finish(),
            Self::External(_) => f.debug_tuple("External").finish_non_exhaustive(),
        }
    }
}

#[derive(Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(InfallibleActionTrait)]
pub enum KnownInfallibleAction {
    EnterStance(EnterStanceAction),
    ExitStance(ExitStanceAction),
    StanceSmite(StanceSmiteAction),
    DoubleHitFollowupAction(DoubleHitFollowupAction),
    ForwardHeavyFollowupAction(ForwardHeavyFollowupAction),
    TrackingFollowupAction(TrackingFollowupAction),
    GoingAction(GoingAction),
    External(Rc<dyn SerializeInfallibleActionTrait>),
}

impl Debug for KnownInfallibleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnterStance(arg0) => f.debug_tuple("EnterStance").field(arg0).finish(),
            Self::ExitStance(arg0) => f.debug_tuple("ExitStance").field(arg0).finish(),
            Self::StanceSmite(arg0) => f.debug_tuple("StanceSmite").field(arg0).finish(),
            Self::DoubleHitFollowupAction(arg0) => f
                .debug_tuple("DoubleHitFollowupAction")
                .field(arg0)
                .finish(),
            Self::ForwardHeavyFollowupAction(arg0) => f
                .debug_tuple("ForwardHeavyFollowupAction")
                .field(arg0)
                .finish(),
            Self::TrackingFollowupAction(arg0) => {
                f.debug_tuple("TrackingFollowupAction").field(arg0).finish()
            }
            Self::GoingAction(arg0) => f.debug_tuple("GoingAction").field(arg0).finish(),
            Self::External(_) => f.debug_tuple("External").finish_non_exhaustive(),
        }
    }
}
