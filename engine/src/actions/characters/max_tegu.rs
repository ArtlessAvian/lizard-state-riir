use std::rc::Rc;

use rkyv::Archive;
use rkyv::Archived;
use rkyv::Deserialize;
use rkyv::Infallible;
use rkyv::Serialize;
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;

use super::super::events::AttackHitEvent;
use super::super::events::StartAttackEvent;
use super::super::CommandTrait;
use super::super::DirectionActionTrait;
use super::super::FloorEvent;
use crate::actions::public::StepAction;
use crate::actions::utils::DelayCommand;
use crate::actions::utils::TakeKnockbackUtil;
use crate::actions::DeserializeCommandTrait;
use crate::actions::SerializeCommandTrait;
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
#[derive(Debug)]
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
                    queued_command: Rc::new(ForwardHeavyFollowup {
                        dir: self.dir,
                        subject_id: self.subject_id,
                    }),
                    turns: 0, // The step already takes a turn.
                    event: None,
                }
                .do_action(&floor)
            })
    }
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug, TypeName))]
struct ForwardHeavyFollowup {
    dir: RelativePosition,
    subject_id: EntityId,
}

#[archive_dyn(deserialize)]
impl CommandTrait for ForwardHeavyFollowup {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        BorrowedFloorUpdate::new(floor)
            .bind(|floor| {
                let mut subject_update = floor.entities[self.subject_id].clone();
                subject_update.state = EntityState::Ok {
                    next_turn: floor.get_current_turn(),
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

// TODO: Figure out how to resolve `private_interfaces` warning. Maybe commands should be public but not constructable?
#[allow(private_interfaces)]
impl CommandTrait for Archived<ForwardHeavyFollowup> {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        // Deserialize and do. Not zero-copy, but whatever.
        Deserialize::<ForwardHeavyFollowup, _>::deserialize(self, &mut Infallible)
            .unwrap()
            .do_action(floor)
    }
}

// Steps forward and sweeps at the start of next turn.
#[derive(Debug)]
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
            queued_command: Rc::new(TrackingFollowup {
                tracking_id,
                subject_id,
            }),
            turns: 1,
            event: Some(FloorEvent::StartAttack(StartAttackEvent {
                subject: subject_id,
                tile: floor.entities[tracking_id].pos,
            })),
        }))
    }
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug, TypeName))]
struct TrackingFollowup {
    tracking_id: EntityId,
    subject_id: EntityId,
}

#[archive_dyn(deserialize)]
impl CommandTrait for TrackingFollowup {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        BorrowedFloorUpdate::new(floor)
            .bind(|floor| {
                let mut subject_update = floor.entities[self.subject_id].clone();
                subject_update.state = EntityState::Ok {
                    next_turn: floor.get_current_turn() + 2,
                };
                subject_update.energy -= 1;
                floor.update_entity((self.subject_id, subject_update))
            })
            .bind(|floor| {
                let object_ref = &floor.entities[self.tracking_id];
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
                        .log(event)
                } else {
                    FloorUpdate::new(floor)
                }
            })
    }
}

// TODO: Figure out how to resolve `private_interfaces` warning. Maybe commands should be public but not constructable?
#[allow(private_interfaces)]
impl CommandTrait for Archived<TrackingFollowup> {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        // Deserialize and do. Not zero-copy, but whatever.
        Deserialize::<TrackingFollowup, _>::deserialize(self, &mut Infallible)
            .unwrap()
            .do_action(floor)
    }
}
