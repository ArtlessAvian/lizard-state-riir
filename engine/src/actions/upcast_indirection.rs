// hmm yes i enjoy this code
// TODO: replace when upcasting is stable in rust.

use std::rc::Rc;

use crate::entity::Entity;
use crate::floor::Floor;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

use super::ActionTrait;
use super::CommandTrait;
use super::DirectionActionTrait;
use super::SerializeActionTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeTileActionTrait;
use super::TileActionTrait;

#[derive(Debug, Clone)]
pub struct Upcast<T: ?Sized>(Rc<T>);

impl<T: ?Sized> Upcast<T> {
    pub fn new(contents: Rc<T>) -> Self {
        Upcast(contents)
    }
}

impl ActionTrait for Upcast<dyn SerializeActionTrait> {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
    ) -> Option<Box<dyn CommandTrait>> {
        self.0.verify_action(floor, subject_ref)
    }
}

impl TileActionTrait for Upcast<dyn SerializeTileActionTrait> {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
        tile: AbsolutePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        self.0.verify_action(floor, subject_ref, tile)
    }
}

impl DirectionActionTrait for Upcast<dyn SerializeDirectionActionTrait> {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
        dir: RelativePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        self.0.verify_action(floor, subject_ref, dir)
    }
}
