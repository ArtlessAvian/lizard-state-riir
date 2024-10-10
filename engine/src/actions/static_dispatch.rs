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

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableAction {
    EnterStance(Rc<EnterStanceAction>),
    ExitStance(Rc<ExitStanceAction>),
    External(Rc<dyn SerializeActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableTileAction {
    Tracking(Rc<TrackingAction>),
    External(Rc<dyn SerializeTileActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum SerializableDirectionAction {
    DoubleHit(Rc<DoubleHitAction>),
    ForwardHeavy(Rc<ForwardHeavyAction>),
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
    #[allow(clippy::clone_on_ref_ptr)]
    fn from(val: SerializableAction) -> Self {
        match val {
            SerializableAction::EnterStance(rc) => rc.clone(),
            SerializableAction::ExitStance(rc) => rc.clone(),
            SerializableAction::External(rc) => Rc::new(Upcast::new(rc)),
        }
    }
}

impl From<SerializableTileAction> for Rc<dyn TileActionTrait> {
    fn from(val: SerializableTileAction) -> Self {
        #[allow(clippy::clone_on_ref_ptr)]
        match val {
            SerializableTileAction::Tracking(rc) => rc.clone(),
            SerializableTileAction::External(rc) => Rc::new(Upcast::new(rc)),
        }
    }
}

impl From<SerializableDirectionAction> for Rc<dyn DirectionActionTrait> {
    fn from(val: SerializableDirectionAction) -> Self {
        #[allow(clippy::clone_on_ref_ptr)]
        match val {
            SerializableDirectionAction::DoubleHit(rc) => rc.clone(),
            SerializableDirectionAction::ForwardHeavy(rc) => rc.clone(),
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
