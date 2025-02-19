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
use super::serializable_wrapper::SerializableAction;
use super::SerializeActionTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeInfallibleActionTrait;
use super::SerializeTileActionTrait;
use super::UnaimedAction;

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

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(ActionTrait)]
pub enum KnownAction {
    External(SerializableAction<dyn SerializeActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(TileActionTrait)]
pub enum KnownTileAction {
    Tracking(TrackingAction),
    EnterSmiteStance(EnterSmiteStanceAction),
    External(SerializableAction<dyn SerializeTileActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[enum_dispatch(DirectionActionTrait)]
pub enum KnownDirectionAction {
    DoubleHit(DoubleHitAction),
    ForwardHeavy(ForwardHeavyAction),
    External(SerializableAction<dyn SerializeDirectionActionTrait>),
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
    External(SerializableAction<dyn SerializeInfallibleActionTrait>),
}
