use std::fmt::Debug;
use std::rc::Rc;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::ActionError;
use super::BoxedCommand;
use super::InfallibleActionTrait;
use super::Never;
use super::SerializeActionTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeInfallibleActionTrait;
use super::SerializeTileActionTrait;
use super::UnaimedAction;
use super::UnaimedMacroTrait;
use super::UnaimedTrait;
use crate::entity::EntityId;
use crate::floor::Floor;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

#[derive(Clone, Archive, Serialize, Deserialize)]
pub enum ErasedUnaimedAction {
    None(Rc<dyn SerializeActionTrait>),
    Tile(Rc<dyn SerializeTileActionTrait>),
    Direction(Rc<dyn SerializeDirectionActionTrait>),
    Infallible(Rc<dyn SerializeInfallibleActionTrait>),
}

impl Debug for ErasedUnaimedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None(_) => f.debug_tuple("None").finish_non_exhaustive(),
            Self::Tile(_) => f.debug_tuple("Tile").finish_non_exhaustive(),
            Self::Direction(_) => f.debug_tuple("Direction").finish_non_exhaustive(),
            Self::Infallible(_) => f.debug_tuple("Infallible").finish_non_exhaustive(),
        }
    }
}

impl From<ErasedUnaimedAction> for UnaimedAction {
    fn from(value: ErasedUnaimedAction) -> Self {
        // mmm yes even more indirection
        match value {
            ErasedUnaimedAction::None(x) => UnaimedAction::None(Rc::new(x)),
            ErasedUnaimedAction::Tile(x) => UnaimedAction::Tile(Rc::new(x)),
            ErasedUnaimedAction::Direction(x) => UnaimedAction::Direction(Rc::new(x)),
            ErasedUnaimedAction::Infallible(x) => UnaimedAction::Infallible(Rc::new(x)),
        }
    }
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
    ) -> Result<BoxedCommand, ActionError> {
        self.as_ref().verify_and_box(floor, subject_id)
    }
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
    ) -> Result<BoxedCommand, ActionError> {
        self.as_ref().verify_and_box(floor, subject_id, tile)
    }
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
    ) -> Result<BoxedCommand, ActionError> {
        self.as_ref().verify_and_box(floor, subject_id, dir)
    }
}

impl UnaimedTrait for Rc<dyn SerializeInfallibleActionTrait> {
    type Target = ();
    type Error = Never;
}

impl UnaimedMacroTrait for Rc<dyn SerializeInfallibleActionTrait> {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        (): (),
    ) -> Result<BoxedCommand, Never> {
        Ok(InfallibleActionTrait::verify_and_box(
            self.as_ref(),
            floor,
            subject_id,
        ))
    }
}
