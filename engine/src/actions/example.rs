use std::borrow::Cow;
use std::rc::Rc;

use rkyv::Archive;
use rkyv::Archived;
use rkyv::Deserialize;
use rkyv::Infallible;
use rkyv::Serialize;
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;

use super::events::AttackHitEvent;
use super::events::PrepareAttackEvent;
use super::events::StartAttackEvent;
use super::utils::DelayCommand;
use super::ActionTrait;
use super::CommandTrait;
use super::DirectionActionTrait;
use super::FloorEvent;
use super::SerializableAction;
use crate::actions::DeserializeActionTrait;
use crate::actions::DeserializeCommandTrait;
use crate::actions::SerializeActionTrait;
use crate::actions::SerializeCommandTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::BorrowedFloorUpdate;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::RelativePosition;

// Hits once, then queues another.
#[derive(Debug)]
pub struct DoubleHitAction;

impl DirectionActionTrait for DoubleHitAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        assert!(floor.entities.contains_id(&subject_id));

        if dir.length() > 1 {
            return None;
        }

        Some(Box::new(DoubleHitCommand { dir, subject_id }))
    }
}

#[derive(Debug)]
struct DoubleHitCommand {
    dir: RelativePosition,
    subject_id: EntityId,
}

impl CommandTrait for DoubleHitCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        BorrowedFloorUpdate::new(floor)
            .map(Cow::Borrowed)
            .log(FloorEvent::StartAttack(StartAttackEvent {
                subject: self.subject_id,
                tile: floor.entities[self.subject_id].pos + self.dir,
            }))
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
                        .map(Cow::Owned),
                )
            })
            .bind(|floor| {
                DelayCommand {
                    subject_id: self.subject_id,
                    queued_command: Rc::new(DoubleHitFollowup {
                        dir: self.dir,
                        subject_id: self.subject_id,
                    }),
                    turns: 1,
                    event: Some(FloorEvent::PrepareAttack(PrepareAttackEvent {
                        subject: self.subject_id,
                        tile: floor.entities[self.subject_id].pos + self.dir,
                    })),
                }
                .do_action(&floor)
            })
    }
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug, TypeName))]
struct DoubleHitFollowup {
    dir: RelativePosition,
    subject_id: EntityId,
}

#[archive_dyn(deserialize)]
impl CommandTrait for DoubleHitFollowup {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        BorrowedFloorUpdate::new(floor)
            .map(Cow::Borrowed)
            .log(FloorEvent::StartAttack(StartAttackEvent {
                subject: self.subject_id,
                tile: floor.entities[self.subject_id].pos + self.dir,
            }))
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
                        .map(Cow::Owned),
                )
            })
            .bind(|floor| {
                let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
                subject_clone.state = EntityState::Ok {
                    next_turn: floor.get_current_turn() + 1,
                };
                floor.update_entity((self.subject_id, subject_clone))
            })
    }
}

// TODO: Figure out how to resolve `private_interfaces` warning. Maybe commands should be public but not constructable?
#[allow(private_interfaces)]
impl CommandTrait for Archived<DoubleHitFollowup> {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        // Deserialize and do. Not zero-copy, but whatever.
        Deserialize::<DoubleHitFollowup, _>::deserialize(self, &mut Infallible)
            .unwrap()
            .do_action(floor)
    }
}

// Waits a turn, then lets the user do a big attack or exit stance.
#[derive(Debug)]
pub struct EnterStanceAction;

impl ActionTrait for EnterStanceAction {
    fn verify_action(&self, _: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        Some(Box::new(EnterStanceCommand { subject_id }))
    }
}

#[derive(Debug)]
pub struct EnterStanceCommand {
    subject_id: EntityId,
}

impl CommandTrait for EnterStanceCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
        subject_clone.state = EntityState::RestrictedActions {
            next_turn: floor.get_current_turn() + 1,
            restricted_actions: vec![SerializableAction::None(Rc::new(ExitStanceAction))],
        };

        floor.update_entity((self.subject_id, subject_clone))
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug, TypeName))]
pub struct ExitStanceAction;

#[archive_dyn(deserialize)]
impl ActionTrait for ExitStanceAction {
    fn verify_action(&self, _: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        Some(Box::new(ExitStanceCommand { subject_id }))
    }
}

impl ActionTrait for Archived<ExitStanceAction> {
    fn verify_action(&self, _: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        Some(Box::new(ExitStanceCommand { subject_id }))
    }
}

#[derive(Debug)]
pub struct ExitStanceCommand {
    subject_id: EntityId,
}

impl CommandTrait for ExitStanceCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
        subject_clone.state = EntityState::Ok {
            next_turn: floor.get_current_turn(),
        };

        floor.update_entity((self.subject_id, subject_clone))
    }
}

#[cfg(test)]
#[test]
fn double_hit() {
    use crate::entity::EntityState;
    use crate::floor::TurntakingError;
    use crate::positional::AbsolutePosition;
    use crate::strategy::Strategy;

    let mut update = FloorUpdate::new(Floor::new());
    let player_id;
    let other_id;
    (update, player_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            state: EntityState::Ok { next_turn: 0 },
            pos: AbsolutePosition::new(0, 0),
            health: 0,
            max_energy: 0,
            energy: 0,
            strategy: Strategy::Null,
            is_player_controlled: true,
            is_player_friendly: false,
            payload: String::default(),
        })
    });
    (update, other_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            state: EntityState::Ok { next_turn: 100 },
            pos: AbsolutePosition::new(1, 0),
            health: 0,
            max_energy: 0,
            energy: 0,
            strategy: Strategy::Null,
            is_player_controlled: true,
            is_player_friendly: false,
            payload: String::default(),
        })
    });
    update = update
        .bind(|floor| {
            DoubleHitAction
                .verify_action(&floor, player_id, RelativePosition::new(1, 0))
                .unwrap()
                .do_action(&floor)
        })
        .bind(|floor| floor.take_npc_turn().unwrap()); // Second hit.

    let (contents, log) = update.into_both();
    assert_eq!(
        contents.take_npc_turn().err(),
        Some(TurntakingError::PlayerTurn { who: player_id })
    );
    assert_eq!(
        log.into_iter()
            .filter(|x| matches!(x, FloorEvent::StartAttack(_) | FloorEvent::AttackHit(_)))
            .collect::<Vec<FloorEvent>>(),
        vec![
            FloorEvent::StartAttack(StartAttackEvent {
                subject: player_id,
                tile: AbsolutePosition::new(1, 0)
            }),
            FloorEvent::AttackHit(AttackHitEvent {
                subject: player_id,
                target: other_id,
                damage: 1,
            }),
            FloorEvent::StartAttack(StartAttackEvent {
                subject: player_id,
                tile: AbsolutePosition::new(1, 0)
            }),
            FloorEvent::AttackHit(AttackHitEvent {
                subject: player_id,
                target: other_id,
                damage: 1,
            }),
        ]
    );
}
