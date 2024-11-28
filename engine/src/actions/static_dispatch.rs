#![allow(private_interfaces)]
use std::rc::Rc;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::characters::max_tegu::ForwardHeavyAction;
use super::characters::max_tegu::ForwardHeavyFollowup;
use super::characters::max_tegu::TrackingAction;
use super::characters::max_tegu::TrackingFollowup;
use super::example::DoubleHitAction;
use super::example::DoubleHitFollowup;
use super::example::EnterStanceAction;
use super::example::ExitStanceAction;
use super::public::GotoCommand;
use super::upcast_indirection::Upcast;
use super::ActionTrait;
use super::CommandTrait;
use super::DirectionActionTrait;
use super::SerializeActionTrait;
use super::SerializeCommandTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeTileActionTrait;
use super::TileActionTrait;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::AbsolutePosition;

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableAction {
    EnterStance(EnterStanceAction),
    ExitStance(ExitStanceAction),
    External(Rc<dyn SerializeActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableTileAction {
    Tracking(TrackingAction),
    External(Rc<dyn SerializeTileActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableDirectionAction {
    DoubleHit(DoubleHitAction),
    ForwardHeavy(ForwardHeavyAction),
    External(Rc<dyn SerializeDirectionActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableCommand {
    DoubleHitFollowup(DoubleHitFollowup),
    ForwardHeavyFollowup(ForwardHeavyFollowup),
    TrackingFollowup(TrackingFollowup),
    Goto(GotoCommand),
    External(Rc<dyn SerializeCommandTrait>),
}

impl From<SerializableAction> for Rc<dyn ActionTrait> {
    fn from(val: SerializableAction) -> Self {
        match val {
            SerializableAction::EnterStance(x) => Rc::new(x),
            SerializableAction::ExitStance(x) => Rc::new(x),
            SerializableAction::External(rc) => Rc::new(Upcast::new(rc)),
        }
    }
}

impl From<SerializableTileAction> for Rc<dyn TileActionTrait> {
    fn from(val: SerializableTileAction) -> Self {
        match val {
            SerializableTileAction::Tracking(x) => Rc::new(x),
            SerializableTileAction::External(rc) => Rc::new(Upcast::new(rc)),
        }
    }
}

impl From<SerializableDirectionAction> for Rc<dyn DirectionActionTrait> {
    fn from(val: SerializableDirectionAction) -> Self {
        #[allow(clippy::clone_on_ref_ptr)]
        match val {
            SerializableDirectionAction::DoubleHit(x) => Rc::new(x),
            SerializableDirectionAction::ForwardHeavy(x) => Rc::new(x),
            SerializableDirectionAction::External(rc) => Rc::new(Upcast::new(rc)),
        }
    }
}

impl From<SerializableCommand> for Rc<dyn CommandTrait> {
    fn from(val: SerializableCommand) -> Self {
        Rc::new(val)
    }
}

impl CommandTrait for SerializableCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        #[allow(clippy::clone_on_ref_ptr)]
        match self {
            SerializableCommand::DoubleHitFollowup(x) => x.do_action(floor),
            SerializableCommand::ForwardHeavyFollowup(x) => x.do_action(floor),
            SerializableCommand::TrackingFollowup(x) => x.do_action(floor),
            SerializableCommand::Goto(x) => x.do_action(floor),
            SerializableCommand::External(rc) => rc.do_action(floor),
        }
    }

    fn get_tile_hints(&self, floor: &Floor) -> Vec<AbsolutePosition> {
        #[allow(clippy::clone_on_ref_ptr)]
        match self {
            SerializableCommand::DoubleHitFollowup(x) => x.get_tile_hints(floor),
            SerializableCommand::ForwardHeavyFollowup(x) => x.get_tile_hints(floor),
            SerializableCommand::TrackingFollowup(x) => x.get_tile_hints(floor),
            SerializableCommand::Goto(x) => x.get_tile_hints(floor),
            SerializableCommand::External(rc) => rc.get_tile_hints(floor),
        }
    }
}
