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
use super::ActionError;
use super::CommandTrait;
use super::SerializeActionTrait;
use super::SerializeCommandTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeTileActionTrait;
use super::UnaimedAction;
use super::UnaimedMacroTrait;
use super::UnaimedTrait;
use crate::entity::EntityId;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

// Same as above, but more specialized. Has an ugly conversion.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
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
#[archive_attr(derive(Debug))]
#[enum_dispatch(ActionTrait)]
pub enum KnownAction {
    EnterStance(EnterStanceAction),
    ExitStance(ExitStanceAction),
    StanceSmite(StanceSmiteAction),
    External(Rc<dyn SerializeActionTrait>),
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
    ) -> Result<Box<dyn CommandTrait>, ActionError> {
        self.as_ref().verify_and_box(floor, subject_id)
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[enum_dispatch(TileActionTrait)]
pub enum KnownTileAction {
    Tracking(TrackingAction),
    EnterSmiteStance(EnterSmiteStanceAction),
    External(Rc<dyn SerializeTileActionTrait>),
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
    ) -> Result<Box<dyn CommandTrait>, ActionError> {
        self.as_ref().verify_and_box(floor, subject_id, tile)
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[enum_dispatch(DirectionActionTrait)]
pub enum KnownDirectionAction {
    DoubleHit(DoubleHitAction),
    ForwardHeavy(ForwardHeavyAction),
    External(Rc<dyn SerializeDirectionActionTrait>),
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
    ) -> Result<Box<dyn CommandTrait>, ActionError> {
        self.as_ref().verify_and_box(floor, subject_id, dir)
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[enum_dispatch(CommandTrait)]
pub enum KnownCommand {
    DoubleHitFollowup(DoubleHitFollowup),
    ForwardHeavyFollowup(ForwardHeavyFollowup),
    TrackingFollowup(TrackingFollowup),
    Goto(GotoCommand),
    External(Rc<dyn SerializeCommandTrait>),
}

impl CommandTrait for Rc<dyn SerializeCommandTrait> {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        self.as_ref().do_action(floor)
    }
}
