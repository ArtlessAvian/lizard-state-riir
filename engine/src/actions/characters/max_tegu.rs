use std::rc::Rc;

use crate::actions::public::StepAction;
use crate::actions::utils::TakeKnockbackUtil;
use crate::actions::DeserializeCommandTrait;
use crate::actions::SerializeCommandTrait;
use crate::actions::TileActionTrait;
use crate::positional::AbsolutePosition;
use rkyv::Archive;
use rkyv::Archived;
use rkyv::Deserialize;
use rkyv::Infallible;
use rkyv::Serialize;
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;

use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::BorrowedFloorUpdate;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::RelativePosition;

// Decide if super::super is good or not
use super::super::events::AttackHitEvent;
use super::super::events::StartAttackEvent;
use super::super::CommandTrait;
use super::super::DirectionActionTrait;
use super::super::FloorEvent;

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
        let update = self.step.do_action(floor);

        update.bind(|floor| {
            let mut subject_clone: Entity = floor.entities[self.subject_id].clone();
            subject_clone.state = EntityState::Committed {
                next_turn: floor.get_current_turn(),
                queued_command: Rc::new(ForwardHeavyFollowup {
                    dir: self.dir,
                    subject_id: subject_clone.id,
                }),
            };
            floor.update_entity(subject_clone)
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
        let update = BorrowedFloorUpdate::new(floor);
        let update = update.bind(|floor| {
            let mut subject_update = floor.entities[self.subject_id].clone();
            subject_update.state = EntityState::Ok {
                next_turn: floor.get_current_turn(),
            };
            subject_update.energy -= 1;
            let event = FloorEvent::StartAttack(StartAttackEvent {
                subject: self.subject_id,
                tile: subject_update.pos + self.dir,
            });
            floor.update_entity(subject_update).log(event)
        });

        update.bind(|floor| {
            if let Some(&object_index) = floor
                .occupiers
                .get(&(floor.entities[self.subject_id].pos + self.dir))
            {
                let object_ref = &floor.entities[object_index];
                let mut object_clone: Entity = object_ref.clone();
                object_clone.health -= 1;

                let event = FloorEvent::AttackHit(AttackHitEvent {
                    subject: self.subject_id,
                    target: object_clone.id,
                    damage: 1,
                });

                let updateee = floor.update_entity(object_clone).log(event);
                updateee.bind(|floor| {
                    // TODO: Add wallbounce
                    TakeKnockbackUtil {
                        entity: object_index,
                        vector: 3 * self.dir,
                    }
                    .do_action(&floor)
                })
            } else {
                FloorUpdate::new(floor)
            }
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

        let tracking_id = *floor.occupiers.get(&(tile))?;

        if tracking_id == subject_id {
            return None;
        }

        Some(Box::new(TrackingCommand {
            tracking_id,
            subject_id,
        }))
    }
}

#[derive(Debug)]
struct TrackingCommand {
    tracking_id: EntityId,
    subject_id: EntityId,
}

impl CommandTrait for TrackingCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut update = BorrowedFloorUpdate::new(floor);
        update = update.log(FloorEvent::StartAttack(StartAttackEvent {
            subject: self.subject_id,
            tile: floor.entities[self.tracking_id].pos,
        }));

        update.bind(|floor| {
            let mut subject_clone: Entity = floor.entities[self.subject_id].clone();
            subject_clone.state = EntityState::Committed {
                next_turn: floor.get_current_turn(),
                queued_command: Rc::new(TrackingFollowup {
                    tracking_id: self.tracking_id,
                    subject_id: subject_clone.id,
                }),
            };
            floor.update_entity(subject_clone)
        })
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
        let update = BorrowedFloorUpdate::new(floor);
        let update = update.bind(|floor| {
            let mut subject_update = floor.entities[self.subject_id].clone();
            subject_update.state = EntityState::Ok {
                next_turn: floor.get_current_turn() + 2,
            };
            subject_update.energy -= 1;
            floor.update_entity(subject_update)
        });

        update.bind(|floor| {
            let object_ref = &floor.entities[self.tracking_id];
            if object_ref.pos.distance(floor.entities[self.subject_id].pos) <= 2 {
                let mut object_clone = (object_ref).clone();
                object_clone.health -= 1;
                let event = FloorEvent::AttackHit(AttackHitEvent {
                    subject: self.subject_id,
                    target: object_clone.id,
                    damage: 1,
                });
                floor.update_entity(object_clone).log(event)
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
