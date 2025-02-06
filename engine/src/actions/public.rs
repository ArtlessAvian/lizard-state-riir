use std::num::NonZero;
use std::rc::Rc;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use super::events::AttackHitEvent;
use super::events::KnockdownEvent;
use super::events::MoveEvent;
use super::events::StartAttackEvent;
use super::events::WakeupEvent;
use super::utils;
use super::utils::TakeKnockbackUtil;
use super::ActionTrait;
use super::CommandTrait;
use super::DirectionActionTrait;
use super::FloorEvent;
use super::TileActionTrait;
use crate::entity::BatchEntityUpdate;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::BorrowedFloorUpdate;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::pathfinding::PathfindingContext;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;
use crate::writer::Writer;

/// Moves one space.
#[derive(Debug)]
pub struct WaitAction;

impl ActionTrait for WaitAction {
    fn verify_action(&self, floor: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        assert!(floor.entities.contains_id(&subject_id));

        Some(Box::new(WaitCommand { subject_id }))
    }
}

#[derive(Debug)]
struct WaitCommand {
    subject_id: EntityId,
}

impl CommandTrait for WaitCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
        subject_clone.state = EntityState::Ok {
            next_round: floor.get_current_round() + 1,
        };
        subject_clone.energy = i8::min(subject_clone.energy + 1, subject_clone.max_energy);

        floor.update_entity((self.subject_id, subject_clone))
    }
}

/// Moves one space.
#[derive(Debug)]
pub struct StepAction;

impl DirectionActionTrait for StepAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        assert!(floor.entities.contains_id(&subject_id));

        if dir.length() != 1 {
            return None;
        }

        if !floor
            .map
            .is_tile_floor(&(floor.entities[subject_id].pos + dir))
        {
            return None;
        }

        // It is impossible to step on yourself if dir.length() == 1, but I'm
        // leaving the check in.
        // TODO: Decide whether to make impossible (leave code as is)
        // or to waste a turn (check in command, no-op if occupied.)
        if floor
            .occupiers
            .get(floor.entities[subject_id].pos + dir)
            .is_some_and(|x| x != subject_id)
        {
            return None;
        }

        Some(Box::new(StepCommand { dir, subject_id }))
    }
}

#[derive(Debug)]
struct StepCommand {
    dir: RelativePosition,
    subject_id: EntityId,
}

impl CommandTrait for StepCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
        subject_clone.pos = subject_clone.pos + self.dir;
        subject_clone.state = EntityState::Ok {
            next_round: floor.get_current_round() + 1,
        };

        BorrowedFloorUpdate::new(floor)
            .log(FloorEvent::Move(MoveEvent {
                subject: self.subject_id,
                tile: subject_clone.pos,
            }))
            .bind(|floor| floor.update_entity((self.subject_id, subject_clone)))
    }
}

/// Does some attack to someone one space away.
///
/// Currently hardcoded to just subtract one health.
#[derive(Debug)]
pub struct BumpAction;

impl DirectionActionTrait for BumpAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        assert!(floor.entities.contains_id(&subject_id));

        if dir.length() != 1 {
            return None;
        }

        Some(Box::new(BumpCommand {
            dir,
            subject_id,
            object_index: floor.occupiers.get(floor.entities[subject_id].pos + dir)?,
            now: floor.get_current_turn()?,
        }))
    }
}

#[derive(Debug)]
struct BumpCommand {
    dir: RelativePosition,
    subject_id: EntityId,
    object_index: EntityId,
    now: (u32, EntityId),
}

impl CommandTrait for BumpCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        BorrowedFloorUpdate::new(floor)
            .log(FloorEvent::StartAttack(StartAttackEvent {
                subject: self.subject_id,
                tile: floor.entities[self.subject_id].pos + self.dir,
            }))
            .log(FloorEvent::AttackHit(AttackHitEvent {
                subject: self.subject_id,
                target: self.object_index,
                damage: 1,
            }))
            .bind(|floor| {
                let mut writer_batch = Writer::new(BatchEntityUpdate::new(&floor.entities));
                writer_batch = writer_batch.bind(|mut batch| {
                    batch
                        .apply_and_insert_writer(self.object_index, |_| {
                            utils::start_juggle(
                                floor,
                                self.object_index,
                                self.now,
                                NonZero::<u32>::new(1).unwrap(),
                            )
                            .map(|mut x| {
                                x.health -= 1;
                                x
                            })
                        })
                        .take(batch)
                });
                writer_batch.apply_and_insert(self.subject_id, |e| Entity {
                    state: EntityState::Ok {
                        next_round: floor.get_current_round() + 1,
                    },
                    ..e.clone()
                });
                writer_batch.bind(|batch| floor.update_entities_batch(batch))
            })
            .bind(|floor| {
                TakeKnockbackUtil {
                    entity: self.object_index,
                    vector: self.dir,
                }
                .do_action(&floor)
            })
    }
}

/// In order, tries to Bump, Walk, or no-op.
///
/// TODO: Maybe move to a submodule.
#[derive(Debug)]
pub struct StepMacroAction;

impl DirectionActionTrait for StepMacroAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_id: EntityId,
        dir: RelativePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        let bump = BumpAction;
        if let Some(command) = bump.verify_action(floor, subject_id, dir) {
            return Some(command);
        }

        let step = StepAction;
        if let Some(command) = step.verify_action(floor, subject_id, dir) {
            return Some(command);
        }

        None
    }
}

#[derive(Debug)]
pub struct GotoAction;

impl TileActionTrait for GotoAction {
    fn verify_action(
        &self,
        _floor: &Floor,
        subject_id: EntityId,
        tile: AbsolutePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        // Pathfind to target.
        Some(Box::new(GotoCommand { tile, subject_id }))
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct GotoCommand {
    pub tile: AbsolutePosition,
    subject_id: EntityId,
}

impl CommandTrait for GotoCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        // TODO: Clean up terrible, terrible nesting.
        let subject_pos = floor.entities[self.subject_id].pos;
        let verify_action = floor
            .map
            .get_step(subject_pos, self.tile)
            .and_then(|target| {
                StepAction.verify_action(
                    floor,
                    self.subject_id,
                    RelativePosition {
                        dx: (target.x - subject_pos.x).clamp(-1, 1),
                        dy: (target.y - subject_pos.y).clamp(-1, 1),
                    },
                )
            })
            .or_else(|| {
                let map = floor.map.clone();
                let entities = floor.entities.clone();
                let context = Rc::clone(&map.pathfinder);
                let pathfinder = context.get()?.borrow();
                let mut step_around = PathfindingContext::new(
                    Box::new(move |pos| {
                        !map.get_tile(&pos).is_tile_floor()
                            || entities
                                .iter()
                                .any(|e| e.1.pos == pos && self.subject_id != e.0)
                    }),
                    Box::new(move |start, destination| {
                        pathfinder.get_distance(start, destination).unwrap_or_else(
                            // fallback. heuristic becomes inconsistent (but still admissible)
                            || AbsolutePosition::distance(start, destination),
                        )
                    }),
                );

                if step_around.find_path(subject_pos, self.tile) {
                    step_around
                        .get_step(subject_pos, self.tile)
                        .and_then(|target| {
                            StepAction.verify_action(
                                floor,
                                self.subject_id,
                                RelativePosition {
                                    dx: (target.x - subject_pos.x).clamp(-1, 1),
                                    dy: (target.y - subject_pos.y).clamp(-1, 1),
                                },
                            )
                        })
                } else {
                    None
                }
            });

        match verify_action {
            None => {
                // Give up immediately, a failed step is not retryable.
                let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
                subject_clone.state = EntityState::Ok {
                    next_round: floor.get_current_round(),
                };
                floor.update_entity((self.subject_id, subject_clone))
            }
            Some(command) => command
                .do_action(floor) // (force indent)
                .bind(|floor| {
                    let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
                    subject_clone.state = if floor.entities[self.subject_id].pos == self.tile {
                        EntityState::Ok {
                            next_round: subject_clone
                                .get_next_round()
                                .expect("This entity just took a step so it has a next turn."),
                        }
                    } else {
                        EntityState::ConfirmCommand {
                            next_round: subject_clone
                                .get_next_round()
                                .expect("This entity just took a step so it has a next turn."),
                            to_confirm: self.clone().into(),
                        }
                    };
                    floor.update_entity((self.subject_id, subject_clone))
                }),
        }
    }
}

#[derive(Debug)]
pub struct TryToStandUpAction;

impl ActionTrait for TryToStandUpAction {
    fn verify_action(&self, floor: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        match floor.entities[subject_id].state {
            EntityState::Knockdown {
                next_round: current_round,
            } => Some(Box::new(TryToStandUpCommand {
                subject_id,
                now: current_round,
            })),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct TryToStandUpCommand {
    subject_id: EntityId,
    now: u32,
}

impl CommandTrait for TryToStandUpCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        if floor
            .occupiers
            .get(floor.entities[self.subject_id].pos)
            .is_some()
        {
            let mut clone = floor.entities[self.subject_id].clone();
            clone.state = EntityState::Knockdown {
                next_round: self.now + 1,
            };
            floor.update_entity((self.subject_id, clone))
        } else {
            let mut clone = floor.entities[self.subject_id].clone();
            clone.state = EntityState::Ok {
                next_round: self.now,
            };
            floor
                .update_entity((self.subject_id, clone))
                .log(FloorEvent::Wakeup(WakeupEvent {
                    subject: self.subject_id,
                }))
        }
    }
}

#[derive(Debug)]
pub struct KnockdownAfterJuggleAction;

impl ActionTrait for KnockdownAfterJuggleAction {
    fn verify_action(&self, floor: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        match floor.entities[subject_id].state {
            EntityState::Hitstun { next_round, .. } => {
                Some(Box::new(KnockdownAfterJuggleCommand {
                    subject_id,
                    now: next_round,
                }))
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct KnockdownAfterJuggleCommand {
    subject_id: EntityId,
    now: u32,
}

impl CommandTrait for KnockdownAfterJuggleCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut clone = floor.entities[self.subject_id].clone();
        clone.state = EntityState::Knockdown {
            next_round: self.now + 1,
        };
        floor
            .update_entity((self.subject_id, clone))
            .log(FloorEvent::KnockdownEvent(KnockdownEvent {
                subject: self.subject_id,
            }))
    }
}

#[cfg(test)]
mod test {
    use crate::actions::events::AttackHitEvent;
    use crate::actions::events::FloorEvent;
    use crate::actions::events::StartAttackEvent;
    use crate::actions::public::BumpAction;
    use crate::actions::public::GotoAction;
    use crate::actions::CommandTrait;
    use crate::actions::DirectionActionTrait;
    use crate::actions::TileActionTrait;
    use crate::entity::Entity;
    use crate::entity::EntityState;
    use crate::floor::Floor;
    use crate::floor::FloorUpdate;
    use crate::positional::AbsolutePosition;
    use crate::positional::RelativePosition;

    #[test]
    fn bump_test() {
        let update = FloorUpdate::new(Floor::new_minimal());
        let (update, player_id) = update
            .bind(|floor| {
                floor.add_entity(Entity {
                    state: EntityState::Ok { next_round: 0 },
                    pos: AbsolutePosition::new(0, 0),
                    ..Default::default()
                })
            })
            .split_pair();
        let (update, other_id) = update
            .bind(|floor| {
                floor.add_entity(Entity {
                    state: EntityState::Ok { next_round: 0 },
                    pos: AbsolutePosition::new(1, 0),
                    ..Default::default()
                })
            })
            .split_pair();
        let update = update.bind(|floor| {
            BumpAction
                .verify_action(&floor, player_id, RelativePosition::new(1, 0))
                .unwrap()
                .do_action(&floor)
        });

        let (floor, log) = update.into_both();
        dbg!(floor);
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
                })
            ]
        );
    }

    #[test]
    fn goto_test() {
        let update = FloorUpdate::new(Floor::new_minimal());
        let (update, player_id) = update
            .bind(|floor| {
                floor.add_entity(Entity {
                    state: EntityState::Ok { next_round: 0 },
                    pos: AbsolutePosition::new(0, 0),
                    ..Default::default()
                })
            })
            .split_pair();
        let update = update.bind(|floor| {
            GotoAction {}
                .verify_action(&floor, player_id, AbsolutePosition::new(5, 3))
                .unwrap()
                .do_action(&floor)
        });

        let confirm_command = |floor: Floor| match &floor.entities[player_id].state {
            EntityState::ConfirmCommand { to_confirm, .. } => to_confirm.do_action(&floor),
            _ => panic!(
                "Expected ConfirmCommand state. Got value: {:?}",
                floor.entities[player_id].state
            ),
        };

        let update = update
            .bind(confirm_command)
            .bind(confirm_command)
            .bind(confirm_command)
            .bind(confirm_command);

        let (floor, _) = update.into_both();
        assert!(matches!(
            floor.entities[player_id].state,
            EntityState::Ok { next_round: 5 }
        ));
        assert_eq!(floor.entities[player_id].pos, AbsolutePosition::new(5, 3));
    }
}
