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

#[derive(Archive, Serialize, Deserialize)]
pub struct ErasedAction<TraitObject: ?Sized>(pub Rc<TraitObject>);

impl<T: ?Sized> Debug for ErasedAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ErasedAction").finish_non_exhaustive()
    }
}

impl<T: ?Sized> Clone for ErasedAction<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum ErasedUnaimedAction {
    None(ErasedAction<dyn SerializeActionTrait>),
    Tile(ErasedAction<dyn SerializeTileActionTrait>),
    Direction(ErasedAction<dyn SerializeDirectionActionTrait>),
    Infallible(ErasedAction<dyn SerializeInfallibleActionTrait>),
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

impl UnaimedTrait for ErasedAction<dyn SerializeActionTrait> {
    type Target = ();
    type Error = ActionError;
}

impl UnaimedMacroTrait for ErasedAction<dyn SerializeActionTrait> {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        (): (),
    ) -> Result<BoxedCommand, ActionError> {
        self.0.as_ref().verify_and_box(floor, subject_id)
    }
}

impl UnaimedTrait for ErasedAction<dyn SerializeTileActionTrait> {
    type Target = AbsolutePosition;
    type Error = ActionError;
}

impl UnaimedMacroTrait for ErasedAction<dyn SerializeTileActionTrait> {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Result<BoxedCommand, ActionError> {
        self.0.as_ref().verify_and_box(floor, subject_id, tile)
    }
}

impl UnaimedTrait for ErasedAction<dyn SerializeDirectionActionTrait> {
    type Target = RelativePosition;
    type Error = ActionError;
}

impl UnaimedMacroTrait for ErasedAction<dyn SerializeDirectionActionTrait> {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Result<BoxedCommand, ActionError> {
        self.0.as_ref().verify_and_box(floor, subject_id, dir)
    }
}

impl UnaimedTrait for ErasedAction<dyn SerializeInfallibleActionTrait> {
    type Target = ();
    type Error = Never;
}

impl UnaimedMacroTrait for ErasedAction<dyn SerializeInfallibleActionTrait> {
    fn verify_and_box(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        (): (),
    ) -> Result<BoxedCommand, Never> {
        Ok(InfallibleActionTrait::verify_and_box(
            self.0.as_ref(),
            floor,
            subject_id,
        ))
    }
}
