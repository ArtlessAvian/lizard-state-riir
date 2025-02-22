use std::fmt::Debug;
use std::rc::Rc;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;
use rkyv_dyn::SerializeDyn;

use super::ActionError;
use super::BoxedCommand;
use super::InfallibleActionTrait;
use super::Never;
use super::SerializeActionTrait;
use super::SerializeDirectionActionTrait;
use super::SerializeInfallibleActionTrait;
use super::SerializeTileActionTrait;
use super::UnaimedMacroTrait;
use super::UnaimedTrait;
use crate::entity::EntityId;
use crate::floor::Floor;
use crate::lazyrc::LazyRc;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

#[derive(Archive, Serialize, Deserialize)]
pub struct SerializableAction<TraitObject: ?Sized + SerializeDyn>(pub Rc<TraitObject>);

impl<T: ?Sized + SerializeDyn> Debug for SerializableAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SerializableExternal")
            .finish_non_exhaustive()
    }
}

impl<T: ?Sized + SerializeDyn> Clone for SerializableAction<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl UnaimedTrait for SerializableAction<dyn SerializeActionTrait> {
    type Target = ();
    type Error = ActionError;
}

impl UnaimedMacroTrait for SerializableAction<dyn SerializeActionTrait> {
    fn verify_and_box(
        &self,
        floor: &LazyRc<Floor>,
        subject_id: EntityId,
        (): (),
    ) -> Result<BoxedCommand, ActionError> {
        self.0.as_ref().verify_and_box(floor, subject_id)
    }
}

impl UnaimedTrait for SerializableAction<dyn SerializeTileActionTrait> {
    type Target = AbsolutePosition;
    type Error = ActionError;
}

impl UnaimedMacroTrait for SerializableAction<dyn SerializeTileActionTrait> {
    fn verify_and_box(
        &self,
        floor: &LazyRc<Floor>,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Result<BoxedCommand, ActionError> {
        self.0.as_ref().verify_and_box(floor, subject_id, tile)
    }
}

impl UnaimedTrait for SerializableAction<dyn SerializeDirectionActionTrait> {
    type Target = RelativePosition;
    type Error = ActionError;
}

impl UnaimedMacroTrait for SerializableAction<dyn SerializeDirectionActionTrait> {
    fn verify_and_box(
        &self,
        floor: &LazyRc<Floor>,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Result<BoxedCommand, ActionError> {
        self.0.as_ref().verify_and_box(floor, subject_id, dir)
    }
}

impl UnaimedTrait for SerializableAction<dyn SerializeInfallibleActionTrait> {
    type Target = ();
    type Error = Never;
}

impl UnaimedMacroTrait for SerializableAction<dyn SerializeInfallibleActionTrait> {
    fn verify_and_box(
        &self,
        floor: &LazyRc<Floor>,
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
