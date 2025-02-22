use std::borrow::Cow;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::super::events::AttackHitEvent;
use super::super::events::StartAttackEvent;
use super::super::CommandTrait;
use super::super::FloorEvent;
use crate::actions::events::PrepareAttackEvent;
use crate::actions::known_serializable::KnownDirectionAction;
use crate::actions::known_serializable::KnownTileAction;
use crate::actions::known_serializable::KnownUnaimedAction;
use crate::actions::public::StepAction;
use crate::actions::public::StepCommand;
use crate::actions::utils::DelayCommand;
use crate::actions::utils::TakeKnockbackUtil;
use crate::actions::ActionError;
use crate::actions::Never;
use crate::actions::UnaimedActionTrait;
use crate::actions::UnaimedTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::CowFloorUpdate;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

// Steps forward and sweeps at the start of next turn.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ForwardHeavyAction;

impl UnaimedTrait for ForwardHeavyAction {
    type Target = RelativePosition;
    type Error = ActionError;
}

impl UnaimedActionTrait for ForwardHeavyAction {
    type Command<'a> = ForwardHeavyCommand<'a>;

    fn verify<'a>(
        &self,
        floor: &Cow<'a, Floor>,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Result<Self::Command<'a>, ActionError> {
        if floor.entities[subject_id].energy <= 0 {
            return Err(ActionError::NotEnoughEnergy);
        }
        Ok(ForwardHeavyCommand {
            step: StepAction.verify(floor, subject_id, dir)?,
            dir,
            subject_id,
        })
    }
}

impl From<ForwardHeavyAction> for KnownUnaimedAction {
    fn from(value: ForwardHeavyAction) -> Self {
        KnownUnaimedAction::Direction(KnownDirectionAction::ForwardHeavy(value))
    }
}

#[derive(Debug)]
pub struct ForwardHeavyCommand<'a> {
    step: StepCommand<'a>,
    dir: RelativePosition,
    subject_id: EntityId,
}

impl CommandTrait for ForwardHeavyCommand<'_> {
    fn do_action(self) -> FloorUpdate {
        self.step
            .do_action() // (forcing indented formatting)
            .bind(|floor| {
                DelayCommand {
                    parsed_floor: Cow::Owned(floor),
                    subject_id: self.subject_id,
                    queued_command: ForwardHeavyFollowupAction { dir: self.dir }.into(),
                    turns: 0, // The step already takes a turn.
                    event: None,
                }
                .do_action()
                .peek_and_log(|floor| {
                    FloorEvent::PrepareAttack(PrepareAttackEvent {
                        subject: self.subject_id,
                        tile: floor.entities[self.subject_id].pos + self.dir,
                    })
                })
            })
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ForwardHeavyFollowupAction {
    dir: RelativePosition,
}

impl UnaimedTrait for ForwardHeavyFollowupAction {
    type Target = ();
    type Error = Never;
}

impl UnaimedActionTrait for ForwardHeavyFollowupAction {
    type Command<'a> = ForwardHeavyFollowup<'a>;

    fn verify<'a>(
        &self,
        floor: &Cow<'a, Floor>,
        subject_id: EntityId,
        (): (),
    ) -> Result<Self::Command<'a>, Self::Error> {
        Ok(ForwardHeavyFollowup {
            parsed_floor: floor.clone(),
            dir: self.dir,
            subject_id,
        })
    }
}

#[derive(Debug)]
pub struct ForwardHeavyFollowup<'a> {
    parsed_floor: Cow<'a, Floor>,
    dir: RelativePosition,
    subject_id: EntityId,
}

impl CommandTrait for ForwardHeavyFollowup<'_> {
    fn do_action(self) -> FloorUpdate {
        CowFloorUpdate::new(self.parsed_floor)
            .bind(|floor| {
                let mut subject_update = floor.entities[self.subject_id].clone();
                subject_update.state = EntityState::Ok {
                    next_round: floor.get_current_round(),
                };
                subject_update.energy -= 1;
                let event = FloorEvent::StartAttack(StartAttackEvent {
                    subject: self.subject_id,
                    tile: subject_update.pos + self.dir,
                });
                floor
                    .update_entity((self.subject_id, subject_update))
                    .log(event)
            })
            .bind_if_some(
                |floor| {
                    floor
                        .occupiers
                        .get(floor.entities[self.subject_id].pos + self.dir)
                },
                |floor, object_index| {
                    let object_ref = &floor.entities[object_index];
                    let mut object_clone: Entity = object_ref.clone();
                    object_clone.health -= 1;

                    floor
                        .update_entity((object_index, object_clone))
                        .log(FloorEvent::AttackHit(AttackHitEvent {
                            subject: self.subject_id,
                            target: object_index,
                            damage: 1,
                        }))
                        .bind(|floor| {
                            TakeKnockbackUtil {
                                parsed_floor: Cow::Owned(floor),
                                entity: object_index,
                                vector: 3 * self.dir,
                            }
                            .do_action()
                        })
                },
            )
    }
}

// Steps forward and sweeps at the start of next turn.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct TrackingAction;

impl UnaimedTrait for TrackingAction {
    type Target = AbsolutePosition;
    type Error = ActionError;
}

impl UnaimedActionTrait for TrackingAction {
    type Command<'a> = DelayCommand<'a>;

    fn verify<'a>(
        &self,
        floor: &Cow<'a, Floor>,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Result<Self::Command<'a>, ActionError> {
        if floor.entities[subject_id].energy <= 0 {
            return Err(ActionError::NotEnoughEnergy);
        }

        let tracking_id = floor
            .occupiers
            .get(tile)
            .ok_or(ActionError::InvalidTarget)?;

        if tracking_id == subject_id {
            return Err(ActionError::InvalidTarget);
        }

        Ok(DelayCommand {
            parsed_floor: floor.clone(),
            subject_id,
            queued_command: TrackingFollowupAction { tracking_id }.into(),
            turns: 1,
            event: Some(FloorEvent::PrepareAttack(PrepareAttackEvent {
                subject: subject_id,
                tile: floor.entities[tracking_id].pos,
            })),
        })
    }
}

impl From<TrackingAction> for KnownUnaimedAction {
    fn from(value: TrackingAction) -> Self {
        KnownUnaimedAction::Tile(KnownTileAction::Tracking(value))
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct TrackingFollowupAction {
    tracking_id: EntityId,
}

impl UnaimedTrait for TrackingFollowupAction {
    type Target = ();
    type Error = Never;
}

impl UnaimedActionTrait for TrackingFollowupAction {
    type Command<'a> = TrackingFollowup<'a>;

    fn verify<'a>(
        &self,
        floor: &Cow<'a, Floor>,
        subject_id: EntityId,
        (): (),
    ) -> Result<Self::Command<'a>, Self::Error> {
        Ok(TrackingFollowup {
            parsed_floor: floor.clone(),
            tracking_id: self.tracking_id,
            subject_id,
        })
    }
}

#[derive(Debug)]
pub struct TrackingFollowup<'a> {
    parsed_floor: Cow<'a, Floor>,
    tracking_id: EntityId,
    subject_id: EntityId,
}

impl CommandTrait for TrackingFollowup<'_> {
    fn do_action(self) -> FloorUpdate {
        CowFloorUpdate::new(self.parsed_floor)
            .bind(|floor| {
                let mut subject_update = floor.entities[self.subject_id].clone();
                subject_update.state = EntityState::Ok {
                    next_round: floor.get_current_round() + 2,
                };
                subject_update.energy -= 1;
                floor.update_entity((self.subject_id, subject_update))
            })
            .bind(|floor| {
                let object_ref = &floor.entities[self.tracking_id];
                let start_attack = FloorEvent::StartAttack(StartAttackEvent {
                    subject: self.subject_id,
                    tile: object_ref.pos,
                });

                if object_ref.pos.distance(floor.entities[self.subject_id].pos) <= 2 {
                    let mut object_clone = (object_ref).clone();
                    object_clone.health -= 1;
                    let event = FloorEvent::AttackHit(AttackHitEvent {
                        subject: self.subject_id,
                        target: self.tracking_id,
                        damage: 1,
                    });
                    floor
                        .update_entity((self.tracking_id, object_clone))
                        .log(start_attack)
                        .log(event)
                } else {
                    FloorUpdate::new(floor).log(start_attack)
                }
            })
    }
}
