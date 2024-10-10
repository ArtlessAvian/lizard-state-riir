#![allow(private_interfaces)]
use std::rc::Rc;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::characters::max_tegu::ForwardHeavyFollowup;
use super::characters::max_tegu::TrackingFollowup;
use super::example::DoubleHitFollowup;
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

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableAction {
    External(Rc<dyn SerializeActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableTileAction {
    External(Rc<dyn SerializeTileActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableDirectionAction {
    External(Rc<dyn SerializeDirectionActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableCommand {
    DoubleHitFollowup(Rc<DoubleHitFollowup>),
    ForwardHeavyFollowup(Rc<ForwardHeavyFollowup>),
    TrackingFollowup(Rc<TrackingFollowup>),
    GotoCommand(Rc<GotoCommand>),
    External(Rc<dyn SerializeCommandTrait>),
}

impl From<SerializableAction> for Rc<dyn ActionTrait> {
    fn from(val: SerializableAction) -> Self {
        match val {
            SerializableAction::External(rc) => Rc::new(Upcast::new(rc)),
        }
    }
}

impl From<SerializableTileAction> for Rc<dyn TileActionTrait> {
    fn from(val: SerializableTileAction) -> Self {
        match val {
            SerializableTileAction::External(rc) => Rc::new(Upcast::new(rc)),
        }
    }
}

impl From<SerializableDirectionAction> for Rc<dyn DirectionActionTrait> {
    fn from(val: SerializableDirectionAction) -> Self {
        match val {
            SerializableDirectionAction::External(rc) => Rc::new(Upcast::new(rc)),
        }
    }
}

impl From<SerializableCommand> for Rc<dyn CommandTrait> {
    fn from(val: SerializableCommand) -> Self {
        #[allow(clippy::clone_on_ref_ptr)]
        match val {
            SerializableCommand::DoubleHitFollowup(rc) => rc.clone(),
            SerializableCommand::ForwardHeavyFollowup(rc) => rc.clone(),
            SerializableCommand::TrackingFollowup(rc) => rc.clone(),
            SerializableCommand::GotoCommand(rc) => rc.clone(),
            SerializableCommand::External(rc) => Rc::new(Upcast::new(rc)),
        }
    }
}

impl CommandTrait for SerializableCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        Into::<Rc<dyn CommandTrait>>::into(self.clone()).do_action(floor)
    }
}
