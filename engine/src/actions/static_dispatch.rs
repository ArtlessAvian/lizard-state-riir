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
use super::ActionTrait;
use super::CommandTrait;
use super::DirectionActionTrait;
use super::SerializeActionTrait;
use super::SerializeCommandTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeTileActionTrait;
use super::TileActionTrait;
use crate::entity::EntityId;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[enum_dispatch]
pub enum SerializableAction {
    EnterStance(EnterStanceAction),
    ExitStance(ExitStanceAction),
    StanceSmite(StanceSmiteAction),
    External(Rc<dyn SerializeActionTrait>),
}

impl ActionTrait for Rc<dyn SerializeActionTrait> {
    fn verify_action(&self, floor: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        self.as_ref().verify_action(floor, subject_id)
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[enum_dispatch]
pub enum SerializableTileAction {
    Tracking(TrackingAction),
    EnterSmiteStance(EnterSmiteStanceAction),
    External(Rc<dyn SerializeTileActionTrait>),
}

impl TileActionTrait for Rc<dyn SerializeTileActionTrait> {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        self.as_ref().verify_action(floor, subject_id, tile)
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[enum_dispatch]
pub enum SerializableDirectionAction {
    DoubleHit(DoubleHitAction),
    ForwardHeavy(ForwardHeavyAction),
    External(Rc<dyn SerializeDirectionActionTrait>),
}

impl DirectionActionTrait for Rc<dyn SerializeDirectionActionTrait> {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        self.as_ref().verify_action(floor, subject_id, dir)
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[enum_dispatch]
pub enum SerializableCommand {
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
