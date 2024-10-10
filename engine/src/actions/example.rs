use std::borrow::Cow;
use std::rc::Rc;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::events::AttackHitEvent;
use super::events::PrepareAttackEvent;
use super::events::StartAttackEvent;
use super::static_dispatch::SerializableAction;
use super::static_dispatch::SerializableCommand;
use super::static_dispatch::SerializableDirectionAction;
use super::utils::DelayCommand;
use super::ActionTrait;
use super::CommandTrait;
use super::DirectionActionTrait;
use super::FloorEvent;
use super::SerializableUnaimedAction;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::BorrowedFloorUpdate;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::RelativePosition;

// Hits once, then queues another.
#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
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

impl From<DoubleHitAction> for SerializableUnaimedAction {
    fn from(value: DoubleHitAction) -> Self {
        SerializableUnaimedAction::Direction(SerializableDirectionAction::DoubleHit(Rc::new(value)))
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
                    queued_command: DoubleHitFollowup {
                        dir: self.dir,
                        subject_id: self.subject_id,
                    }
                    .into(),
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
#[archive_attr(derive(Debug))]
pub(crate) struct DoubleHitFollowup {
    dir: RelativePosition,
    subject_id: EntityId,
}

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

impl From<DoubleHitFollowup> for SerializableCommand {
    fn from(val: DoubleHitFollowup) -> Self {
        SerializableCommand::DoubleHitFollowup(Rc::new(val))
    }
}

// Waits a turn, then lets the user do a big attack or exit stance.
#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct EnterStanceAction;

impl ActionTrait for EnterStanceAction {
    fn verify_action(&self, _: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        Some(Box::new(EnterStanceCommand { subject_id }))
    }
}

impl From<EnterStanceAction> for SerializableUnaimedAction {
    fn from(value: EnterStanceAction) -> Self {
        SerializableUnaimedAction::None(SerializableAction::EnterStance(Rc::new(value)))
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
            restricted_actions: vec![ExitStanceAction.into()],
        };

        floor.update_entity((self.subject_id, subject_clone))
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct ExitStanceAction;

impl ActionTrait for ExitStanceAction {
    fn verify_action(&self, _: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        Some(Box::new(ExitStanceCommand { subject_id }))
    }
}

impl From<ExitStanceAction> for SerializableUnaimedAction {
    fn from(value: ExitStanceAction) -> Self {
        SerializableUnaimedAction::None(SerializableAction::ExitStance(Rc::new(value)))
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

    let mut update = FloorUpdate::new(Floor::new());
    let player_id;
    let other_id;
    (update, player_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            state: EntityState::Ok { next_turn: 0 },
            pos: AbsolutePosition::new(0, 0),
            is_player_controlled: true,
            ..Default::default()
        })
    });
    (update, other_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            state: EntityState::Ok { next_turn: 100 },
            pos: AbsolutePosition::new(1, 0),
            ..Default::default()
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
