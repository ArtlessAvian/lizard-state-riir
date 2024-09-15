use std::rc::Rc;

use rkyv::Archive;
use rkyv::Archived;
use rkyv::Deserialize;
use rkyv::Infallible;
use rkyv::Serialize;
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;

use super::events::AttackHitEvent;
use super::events::MoveEvent;
use super::events::StartAttackEvent;
use super::utils::TakeKnockbackUtil;
use super::ActionTrait;
use super::CommandTrait;
use super::DirectionActionTrait;
use super::FloorEvent;
use super::TileActionTrait;
use crate::actions::DeserializeCommandTrait;
use crate::actions::SerializeCommandTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::BorrowedFloorUpdate;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

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
            next_turn: floor.get_current_turn() + 1,
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
            next_turn: floor.get_current_turn() + 1,
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
        }))
    }
}

#[derive(Debug)]
struct BumpCommand {
    dir: RelativePosition,
    subject_id: EntityId,
    object_index: EntityId,
}

impl CommandTrait for BumpCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
        subject_clone.state = EntityState::Ok {
            next_turn: floor.get_current_turn() + 1,
        };

        let object_ref = &floor.entities[self.object_index];
        let mut object_clone: Entity = object_ref.clone();
        object_clone.health -= 1;

        object_clone.state = EntityState::Ok {
            next_turn: subject_clone
                .get_next_turn()
                .expect("bump command subject just took their turn")
                + 1,
        };

        BorrowedFloorUpdate::new(floor)
            .log(FloorEvent::StartAttack(StartAttackEvent {
                subject: self.subject_id,
                tile: subject_clone.pos + self.dir,
            }))
            .log(FloorEvent::AttackHit(AttackHitEvent {
                subject: self.subject_id,
                target: self.object_index,
                damage: 1,
            }))
            .bind(|floor| {
                floor.update_entities(Vec::from([
                    (self.subject_id, subject_clone),
                    (self.object_index, object_clone),
                ]))
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
#[archive_attr(derive(Debug, TypeName))]
pub struct GotoCommand {
    pub tile: AbsolutePosition,
    subject_id: EntityId,
}

#[archive_dyn(deserialize)]
impl CommandTrait for GotoCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
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
            });

        match verify_action {
            None => {
                // Give up immediately.
                // Even when pathfinding is implemented, a failed step should probably mean to stop.
                let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
                subject_clone.state = EntityState::Ok {
                    next_turn: floor.get_current_turn(),
                };
                floor.update_entity((self.subject_id, subject_clone))
            }
            Some(command) => command
                .do_action(floor) // (force indent)
                .bind(|floor| {
                    let mut subject_clone: Entity = (floor.entities[self.subject_id]).clone();
                    subject_clone.state = if floor.entities[self.subject_id].pos == self.tile {
                        EntityState::Ok {
                            next_turn: subject_clone
                                .get_next_turn()
                                .expect("This entity just took a step so it has a next turn."),
                        }
                    } else {
                        EntityState::ConfirmCommand {
                            next_turn: subject_clone
                                .get_next_turn()
                                .expect("This entity just took a step so it has a next turn."),
                            to_confirm: Rc::new(self.clone()),
                        }
                    };
                    floor.update_entity((self.subject_id, subject_clone))
                }),
        }
    }
}

// TODO: Figure out how to resolve `private_interfaces` warning. Maybe commands should be public but not constructable?
#[allow(private_interfaces)]
impl CommandTrait for Archived<GotoCommand> {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        // Deserialize and do. Not zero-copy, but whatever.
        Deserialize::<GotoCommand, _>::deserialize(self, &mut Infallible)
            .unwrap()
            .do_action(floor)
    }
}

#[derive(Debug)]
pub struct TryToStandUpAction;

impl ActionTrait for TryToStandUpAction {
    fn verify_action(&self, floor: &Floor, subject_id: EntityId) -> Option<Box<dyn CommandTrait>> {
        match floor.entities[subject_id].state {
            EntityState::Knockdown { next_turn } => Some(Box::new(TryToStandUpCommand {
                subject_id,
                now: next_turn,
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
                next_turn: self.now + 1,
            };
            floor.update_entity((self.subject_id, clone))
        } else {
            let mut clone = floor.entities[self.subject_id].clone();
            clone.state = EntityState::Ok {
                next_turn: self.now,
            };
            floor.update_entity((self.subject_id, clone))
        }
    }
}

#[cfg(test)]
#[test]
fn bump_test() {
    use crate::entity::EntityState;
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
            is_player_controlled: false,
            is_player_friendly: false,
        })
    });
    (update, other_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            state: EntityState::Ok { next_turn: 0 },
            pos: AbsolutePosition::new(1, 0),
            health: 0,
            max_energy: 0,
            energy: 0,
            strategy: Strategy::Null,
            is_player_controlled: false,
            is_player_friendly: false,
        })
    });
    update = update.bind(|floor| {
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

#[cfg(test)]
#[test]
fn goto_test() {
    use crate::entity::EntityState;
    use crate::positional::AbsolutePosition;
    use crate::strategy::Strategy;

    let mut update = FloorUpdate::new(Floor::new());
    let player_id;
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
        })
    });
    update = update.bind(|floor| {
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

    update = update
        .bind(confirm_command)
        .bind(confirm_command)
        .bind(confirm_command)
        .bind(confirm_command);

    let (floor, _) = update.into_both();
    assert!(matches!(
        floor.entities[player_id].state,
        EntityState::Ok { next_turn: 5 }
    ));
    assert_eq!(floor.entities[player_id].pos, AbsolutePosition::new(5, 3));
}
