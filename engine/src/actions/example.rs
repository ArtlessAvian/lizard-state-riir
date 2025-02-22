use std::borrow::Cow;
use std::rc::Rc;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::events::AttackHitEvent;
use super::events::PrepareAttackEvent;
use super::events::StartAttackEvent;
use super::known_serializable::KnownDirectionAction;
use super::known_serializable::KnownInfallibleAction;
use super::known_serializable::KnownUnaimedAction;
use super::utils::DelayCommand;
use super::ActionError;
use super::CommandTrait;
use super::FloorEvent;
use super::Never;
use super::UnaimedActionTrait;
use super::UnaimedTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::BorrowedFloorUpdate;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::RelativePosition;

// Hits once, then queues another.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct DoubleHitAction;

impl UnaimedTrait for DoubleHitAction {
    type Target = RelativePosition;
    type Error = ActionError;
}

impl UnaimedActionTrait for DoubleHitAction {
    type Command = DoubleHitCommand;

    fn verify(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Result<Self::Command, ActionError> {
        if !(floor.entities.contains_id(subject_id)) {
            return Err(ActionError::DataMismatch);
        }

        if dir.length() > 1 {
            return Err(ActionError::TargetOutOfRange);
        }

        Ok(DoubleHitCommand {
            parsed_floor: Rc::new(floor.clone()),
            dir,
            subject_id,
        })
    }
}

impl From<DoubleHitAction> for KnownUnaimedAction {
    fn from(value: DoubleHitAction) -> Self {
        KnownUnaimedAction::Direction(KnownDirectionAction::DoubleHit(value))
    }
}

#[derive(Debug)]
pub struct DoubleHitCommand {
    parsed_floor: Rc<Floor>,
    dir: RelativePosition,
    subject_id: EntityId,
}

impl CommandTrait for DoubleHitCommand {
    fn do_action(self) -> FloorUpdate {
        BorrowedFloorUpdate::new(&self.parsed_floor)
            .peek_and_log(|floor| {
                FloorEvent::StartAttack(StartAttackEvent {
                    subject: self.subject_id,
                    tile: floor.entities[self.subject_id].pos + self.dir,
                })
            })
            .map(Cow::Borrowed)
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
                        .map(Cow::Owned)
                },
            )
            .bind(|floor| {
                let pos = floor.entities[self.subject_id].pos;
                DelayCommand {
                    parsed_floor: Rc::new(floor.into_owned()),
                    subject_id: self.subject_id,
                    queued_command: DoubleHitFollowupAction { dir: self.dir }.into(),
                    turns: 1,
                    event: Some(FloorEvent::PrepareAttack(PrepareAttackEvent {
                        subject: self.subject_id,
                        tile: pos + self.dir,
                    })),
                }
                .do_action()
            })
    }
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct DoubleHitFollowupAction {
    dir: RelativePosition,
}

impl UnaimedTrait for DoubleHitFollowupAction {
    type Target = ();
    type Error = Never;
}

impl UnaimedActionTrait for DoubleHitFollowupAction {
    type Command = DoubleHitFollowup;

    fn verify(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        (): (),
    ) -> Result<DoubleHitFollowup, Self::Error> {
        Ok(DoubleHitFollowup {
            parsed_floor: Rc::new(floor.clone()),
            dir: self.dir,
            subject_id,
        })
    }
}

#[derive(Debug)]
pub struct DoubleHitFollowup {
    parsed_floor: Rc<Floor>,
    dir: RelativePosition,
    subject_id: EntityId,
}

impl CommandTrait for DoubleHitFollowup {
    fn do_action(self) -> FloorUpdate {
        BorrowedFloorUpdate::new(&self.parsed_floor)
            .peek_and_log(|floor| {
                FloorEvent::StartAttack(StartAttackEvent {
                    subject: self.subject_id,
                    tile: floor.entities[self.subject_id].pos + self.dir,
                })
            })
            .map(Cow::Borrowed)
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
                        .map(Cow::Owned)
                },
            )
            .bind(|floor| {
                let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
                subject_clone.state = EntityState::Ok {
                    next_round: floor.get_current_round() + 1,
                };
                floor.update_entity((self.subject_id, subject_clone))
            })
    }
}

// Waits a turn, then lets the user do a big attack or exit stance.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct EnterStanceAction;

impl UnaimedTrait for EnterStanceAction {
    type Target = ();
    type Error = Never;
}

impl UnaimedActionTrait for EnterStanceAction {
    type Command = EnterStanceCommand;

    fn verify(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        (): (),
    ) -> Result<Self::Command, Self::Error> {
        Ok(EnterStanceCommand {
            parsed_floor: Rc::new(floor.clone()),
            subject_id,
        })
    }
}

impl From<EnterStanceAction> for KnownUnaimedAction {
    fn from(value: EnterStanceAction) -> Self {
        KnownUnaimedAction::Infallible(KnownInfallibleAction::EnterStance(value))
    }
}

#[derive(Debug)]
pub struct EnterStanceCommand {
    parsed_floor: Rc<Floor>,
    subject_id: EntityId,
}

impl CommandTrait for EnterStanceCommand {
    fn do_action(self) -> FloorUpdate {
        let mut subject_clone: Entity = (self.parsed_floor.entities[self.subject_id]).clone();
        subject_clone.state = EntityState::RestrictedActions {
            next_round: self.parsed_floor.get_current_round() + 1,
            restricted_actions: vec![ExitStanceAction.into()],
        };

        self.parsed_floor
            .update_entity((self.subject_id, subject_clone))
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct ExitStanceAction;

impl UnaimedTrait for ExitStanceAction {
    type Target = ();
    type Error = Never;
}

impl UnaimedActionTrait for ExitStanceAction {
    type Command = ExitStanceCommand;

    fn verify(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        (): Self::Target,
    ) -> Result<ExitStanceCommand, Self::Error> {
        Ok(ExitStanceCommand {
            parsed_floor: Rc::new(floor.clone()),
            subject_id,
        })
    }
}

impl From<ExitStanceAction> for KnownUnaimedAction {
    fn from(value: ExitStanceAction) -> Self {
        KnownUnaimedAction::Infallible(KnownInfallibleAction::ExitStance(value))
    }
}

#[derive(Debug)]
pub struct ExitStanceCommand {
    parsed_floor: Rc<Floor>,
    subject_id: EntityId,
}

impl CommandTrait for ExitStanceCommand {
    fn do_action(self) -> FloorUpdate {
        let mut subject_clone: Entity = (self.parsed_floor.entities[self.subject_id]).clone();
        subject_clone.state = EntityState::Ok {
            next_round: self.parsed_floor.get_current_round(),
        };

        self.parsed_floor
            .update_entity((self.subject_id, subject_clone))
    }
}

#[cfg(test)]
mod test {
    use crate::actions::events::AttackHitEvent;
    use crate::actions::events::FloorEvent;
    use crate::actions::events::StartAttackEvent;
    use crate::actions::example::DoubleHitAction;
    use crate::actions::CommandTrait;
    use crate::actions::UnaimedActionTrait;
    use crate::entity::Entity;
    use crate::floor::Floor;
    use crate::floor::FloorUpdate;
    use crate::positional::RelativePosition;

    #[test]
    fn double_hit() {
        use crate::entity::EntityState;
        use crate::floor::TurntakingError;
        use crate::positional::AbsolutePosition;

        let update = FloorUpdate::new(Floor::new_minimal());
        let (update, player_id) = update
            .bind(|floor| {
                floor.add_entity(Entity {
                    state: EntityState::Ok { next_round: 0 },
                    pos: AbsolutePosition::new(0, 0),
                    is_player_controlled: true,
                    ..Default::default()
                })
            })
            .split_pair();
        let (update, other_id) = update
            .bind(|floor| {
                floor.add_entity(Entity {
                    state: EntityState::Ok { next_round: 100 },
                    pos: AbsolutePosition::new(1, 0),
                    ..Default::default()
                })
            })
            .split_pair();
        let update = update
            .bind(|floor| {
                DoubleHitAction
                    .verify(&floor, player_id, RelativePosition::new(1, 0))
                    .unwrap()
                    .do_action()
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
}
