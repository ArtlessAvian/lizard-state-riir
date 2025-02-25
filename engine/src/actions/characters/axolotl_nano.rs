use std::borrow::Cow;
use std::num::NonZero;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::super::CommandTrait;
use crate::actions::ActionError;
use crate::actions::Never;
use crate::actions::UnaimedActionTrait;
use crate::actions::UnaimedTrait;
use crate::actions::known_serializable::KnownInfallibleAction;
use crate::actions::known_serializable::KnownTileAction;
use crate::actions::known_serializable::KnownUnaimedAction;
use crate::actions::utils::start_juggle;
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
    type Command<'a> = EnterSmiteStanceCommand<'a>;

    fn verify<'a>(
        &self,
        floor: &Cow<'a, Floor>,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Result<Self::Command<'a>, ActionError> {
        Ok(EnterSmiteStanceCommand {
            parsed_floor: floor.clone(),
            subject_id,
            tile,
        })
    }
}

impl From<EnterSmiteStanceAction> for KnownUnaimedAction {
    fn from(value: EnterSmiteStanceAction) -> Self {
        KnownUnaimedAction::Tile(KnownTileAction::EnterSmiteStance(value))
    }
}

#[derive(Debug, Clone)]
pub struct EnterSmiteStanceCommand<'a> {
    parsed_floor: Cow<'a, Floor>,
    subject_id: EntityId,
    tile: AbsolutePosition,
}

impl CommandTrait for EnterSmiteStanceCommand<'_> {
    fn do_action(self) -> FloorUpdate {
        let mut clone = self.parsed_floor.entities[self.subject_id].clone();
        clone.state = EntityState::RestrictedActions {
            next_round: self.parsed_floor.get_current_round() + 1,
            restricted_actions: Vec::from([StanceSmiteAction { tile: self.tile }.into()]),
        };
        self.parsed_floor.update_entity((self.subject_id, clone))
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
    type Command<'a> = StanceSmiteCommand<'a>;

    fn verify<'a>(
        &self,
        floor: &Cow<'a, Floor>,
        subject_id: EntityId,
        (): (),
    ) -> Result<Self::Command<'a>, Self::Error> {
        Ok(StanceSmiteCommand {
            parsed_floor: floor.clone(),
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
pub struct StanceSmiteCommand<'a> {
    parsed_floor: Cow<'a, Floor>,
    subject_id: EntityId,
    tile: AbsolutePosition,
}

impl CommandTrait for StanceSmiteCommand<'_> {
    fn do_action(self) -> FloorUpdate {
        let now = self.parsed_floor.get_current_turn().unwrap();

        let mut clone = self.parsed_floor.entities[self.subject_id].clone();
        clone.state = EntityState::Ok {
            next_round: now.0 + 1,
        };

        self.parsed_floor
            .update_entity((self.subject_id, clone))
            .bind_if_some(
                |floor| floor.occupiers.get(self.tile),
                |floor, hit_id| {
                    let dingus = start_juggle(&floor, hit_id, now, NonZero::new(1).unwrap());
                    dingus.bind(|hit_clone| floor.update_entity((hit_id, hit_clone)))
                },
            )
    }
}
