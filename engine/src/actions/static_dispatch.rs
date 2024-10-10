use std::rc::Rc;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::SerializeActionTrait;
use super::SerializeCommandTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeTileActionTrait;

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
enum SerializableAction {
    External(Rc<dyn SerializeActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
enum SerializableTileAction {
    External(Rc<dyn SerializeTileActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
enum SerializableDirectionAction {
    External(Rc<dyn SerializeDirectionActionTrait>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
enum SerializableCommand {
    External(Rc<dyn SerializeCommandTrait>),
}
