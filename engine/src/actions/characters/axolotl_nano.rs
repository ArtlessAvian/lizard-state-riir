use std::num::NonZero;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::super::CommandTrait;
use crate::actions::known_serializable::KnownInfallibleAction;
use crate::actions::known_serializable::KnownTileAction;
use crate::actions::known_serializable::KnownUnaimedAction;
use crate::actions::utils::start_juggle;
use crate::actions::ActionError;
use crate::actions::Never;
use crate::actions::UnaimedActionTrait;
use crate::actions::UnaimedTrait;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::AbsolutePosition;

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct EnterSmiteStanceAction;

impl UnaimedTrait for EnterSmiteStanceAction {
    type Target = AbsolutePosition;
    type Error = ActionError;
}

impl UnaimedActionTrait for EnterSmiteStanceAction {
    type Command = EnterSmiteStanceCommand;

    fn verify(
        &self,
        _floor: &Floor,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Result<Self::Command, ActionError> {
        Ok(EnterSmiteStanceCommand { subject_id, tile })
    }
}

impl From<EnterSmiteStanceAction> for KnownUnaimedAction {
    fn from(value: EnterSmiteStanceAction) -> Self {
        KnownUnaimedAction::Tile(KnownTileAction::EnterSmiteStance(value))
    }
}

#[derive(Debug, Clone)]
pub struct EnterSmiteStanceCommand {
    subject_id: EntityId,
    tile: AbsolutePosition,
}

impl CommandTrait for EnterSmiteStanceCommand {
    fn do_action(self, floor: Floor) -> FloorUpdate {
        let mut clone = floor.entities[self.subject_id].clone();
        clone.state = EntityState::RestrictedActions {
            next_round: floor.get_current_round() + 1,
            restricted_actions: Vec::from([StanceSmiteAction { tile: self.tile }.into()]),
        };
        floor.update_entity((self.subject_id, clone))
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct StanceSmiteAction {
    tile: AbsolutePosition,
}

impl UnaimedTrait for StanceSmiteAction {
    type Target = ();
    type Error = Never;
}

impl UnaimedActionTrait for StanceSmiteAction {
    type Command = StanceSmiteCommand;

    fn verify(
        &self,
        _floor: &Floor,
        subject_id: EntityId,
        (): (),
    ) -> Result<Self::Command, Self::Error> {
        Ok(StanceSmiteCommand {
            subject_id,
            tile: self.tile,
        })
    }
}

impl From<StanceSmiteAction> for KnownUnaimedAction {
    fn from(value: StanceSmiteAction) -> Self {
        KnownUnaimedAction::Infallible(KnownInfallibleAction::StanceSmite(value))
    }
}

#[derive(Debug, Clone)]
pub struct StanceSmiteCommand {
    subject_id: EntityId,
    tile: AbsolutePosition,
}

impl CommandTrait for StanceSmiteCommand {
    fn do_action(self, floor: Floor) -> FloorUpdate {
        let now = floor.get_current_turn().unwrap();

        let mut clone = floor.entities[self.subject_id].clone();
        clone.state = EntityState::Ok {
            next_round: now.0 + 1,
        };

        floor.update_entity((self.subject_id, clone)).bind_if_some(
            |floor| floor.occupiers.get(self.tile),
            |floor, hit_id| {
                let dingus = start_juggle(&floor, hit_id, now, NonZero::new(1).unwrap());
                dingus.bind(|hit_clone| floor.update_entity((hit_id, hit_clone)))
            },
        )
    }
}
