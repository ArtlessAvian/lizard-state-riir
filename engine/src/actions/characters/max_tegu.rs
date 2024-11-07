use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::super::events::AttackHitEvent;
use super::super::events::StartAttackEvent;
use super::super::CommandTrait;
use super::super::DirectionActionTrait;
use super::super::FloorEvent;
use crate::actions::events::PrepareAttackEvent;
use crate::actions::public::StepAction;
use crate::actions::static_dispatch::SerializableCommand;
use crate::actions::static_dispatch::SerializableDirectionAction;
use crate::actions::static_dispatch::SerializableTileAction;
use crate::actions::utils::DelayCommand;
use crate::actions::utils::TakeKnockbackUtil;
use crate::actions::SerializableUnaimedAction;
use crate::actions::TileActionTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::BorrowedFloorUpdate;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

// Steps forward and sweeps at the start of next turn.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct ForwardHeavyAction;

impl DirectionActionTrait for ForwardHeavyAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        if floor.entities[subject_id].energy <= 0 {
            return None;
        }
        Some(Box::new(ForwardHeavyCommand {
            step: StepAction.verify_action(floor, subject_id, dir)?,
            dir,
            subject_id,
        }))
    }
}

impl From<ForwardHeavyAction> for SerializableUnaimedAction {
    fn from(value: ForwardHeavyAction) -> Self {
        SerializableUnaimedAction::Direction(SerializableDirectionAction::ForwardHeavy(value))
    }
}

#[derive(Debug)]
struct ForwardHeavyCommand {
    step: Box<dyn CommandTrait>,
    dir: RelativePosition,
    subject_id: EntityId,
}

impl CommandTrait for ForwardHeavyCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        self.step
            .do_action(floor) // (forcing indented formatting)
            .bind(|floor| {
                DelayCommand {
                    subject_id: self.subject_id,
                    queued_command: ForwardHeavyFollowup {
                        dir: self.dir,
                        subject_id: self.subject_id,
                    }
                    .into(),
                    turns: 0, // The step already takes a turn.
                    event: None,
                }
                .do_action(&floor)
                .log(FloorEvent::PrepareAttack(PrepareAttackEvent {
                    subject: self.subject_id,
                    tile: floor.entities[self.subject_id].pos + self.dir,
                }))
            })
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub(crate) struct ForwardHeavyFollowup {
    dir: RelativePosition,
    subject_id: EntityId,
}

impl CommandTrait for ForwardHeavyFollowup {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        BorrowedFloorUpdate::new(floor)
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
            .bind_or_noop(|floor| {
                let object_index = floor
                    .occupiers
                    .get(floor.entities[self.subject_id].pos + self.dir)?;

                let object_ref = &floor.entities[object_index];
                let mut object_clone: Entity = object_ref.clone();
                object_clone.health -= 1;

                Some(
                    floor
                        .update_entity((object_index, object_clone))
                        .log(FloorEvent::AttackHit(AttackHitEvent {
                            subject: self.subject_id,
                            target: object_index,
                            damage: 1,
                        }))
                        .bind(|floor| {
                            TakeKnockbackUtil {
                                entity: object_index,
                                vector: 3 * self.dir,
                            }
                            .do_action(&floor)
                        }),
                )
            })
    }
}

impl From<ForwardHeavyFollowup> for SerializableCommand {
    fn from(value: ForwardHeavyFollowup) -> Self {
        SerializableCommand::ForwardHeavyFollowup(value)
    }
}

// Steps forward and sweeps at the start of next turn.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct TrackingAction;

impl TileActionTrait for TrackingAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        if floor.entities[subject_id].energy <= 0 {
            return None;
        }

        let tracking_id = floor.occupiers.get(tile)?;

        if tracking_id == subject_id {
            return None;
        }

        Some(Box::new(DelayCommand {
            subject_id,
            queued_command: TrackingFollowup {
                tracking_id,
                subject_id,
            }
            .into(),
            turns: 1,
            event: Some(FloorEvent::PrepareAttack(PrepareAttackEvent {
                subject: subject_id,
                tile: floor.entities[tracking_id].pos,
            })),
        }))
    }
}

impl From<TrackingAction> for SerializableUnaimedAction {
    fn from(value: TrackingAction) -> Self {
        SerializableUnaimedAction::Tile(SerializableTileAction::Tracking(value))
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub(crate) struct TrackingFollowup {
    tracking_id: EntityId,
    subject_id: EntityId,
}

impl CommandTrait for TrackingFollowup {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        BorrowedFloorUpdate::new(floor)
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

impl From<TrackingFollowup> for SerializableCommand {
    fn from(value: TrackingFollowup) -> Self {
        SerializableCommand::TrackingFollowup(value)
    }
}
