#![allow(private_interfaces)]
use std::rc::Rc;

use enum_dispatch::enum_dispatch;
use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::characters::axolotl_nano::EnterSmiteStanceAction;
use super::characters::axolotl_nano::StanceSmiteAction;
use super::characters::max_tegu::ForwardHeavyAction;
use super::characters::max_tegu::ForwardHeavyFollowup;
use super::characters::max_tegu::TrackingAction;
use super::characters::max_tegu::TrackingFollowup;
use super::example::DoubleHitAction;
use super::example::DoubleHitFollowup;
use super::example::EnterStanceAction;
use super::example::ExitStanceAction;
use super::public::GotoCommand;
use super::UnaimedAction;

// Same as above, but more specialized. Has an ugly conversion.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub enum KnownUnaimedAction {
    None(KnownAction),
    Tile(KnownTileAction),
    Direction(KnownDirectionAction),
}

impl From<KnownUnaimedAction> for UnaimedAction {
    fn from(value: KnownUnaimedAction) -> Self {
        // mmm yes i love indirection
        match value {
            KnownUnaimedAction::None(x) => UnaimedAction::None(Rc::new(x)),
            KnownUnaimedAction::Tile(x) => UnaimedAction::Tile(Rc::new(x)),
            KnownUnaimedAction::Direction(x) => UnaimedAction::Direction(Rc::new(x)),
        }
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[enum_dispatch(ActionTrait)]
pub enum KnownAction {
    EnterStance(EnterStanceAction),
    ExitStance(ExitStanceAction),
    StanceSmite(StanceSmiteAction),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[enum_dispatch(TileActionTrait)]
pub enum KnownTileAction {
    Tracking(TrackingAction),
    EnterSmiteStance(EnterSmiteStanceAction),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[enum_dispatch(DirectionActionTrait)]
pub enum KnownDirectionAction {
    DoubleHit(DoubleHitAction),
    ForwardHeavy(ForwardHeavyAction),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[enum_dispatch(CommandTrait)]
pub enum KnownCommand {
    DoubleHitFollowup(DoubleHitFollowup),
    ForwardHeavyFollowup(ForwardHeavyFollowup),
    TrackingFollowup(TrackingFollowup),
    Goto(GotoCommand),
}
