use std::rc::Rc;

use crate::actions::DeserializeCommandTrait;
use crate::actions::SerializeCommandTrait;
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
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

use super::events::AttackHitEvent;
use super::events::MoveEvent;
use super::events::StartAttackEvent;
use super::utils::TakeKnockbackUtil;
use super::CommandTrait;
use super::DirectionActionTrait;
use super::FloorEvent;
use super::TileActionTrait;

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

        if dir.length() > 1 {
            return None;
        }

        if !floor
            .map
            .is_tile_floor(&(floor.entities[subject_id].pos + dir))
        {
            return None;
        }

        // TODO: Decide whether to make impossible (leave code as is)
        // or to waste a turn (check in command, no-op if occupied.)
        if floor
            .occupiers
            .get(&(floor.entities[subject_id].pos + dir))
            .is_some_and(|x| x != &floor.entities[subject_id].id)
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
        let mut update = BorrowedFloorUpdate::new(floor);

        let mut subject_clone: Entity = (*floor.entities[self.subject_id]).clone();
        subject_clone.pos = subject_clone.pos + self.dir;
        subject_clone.state = EntityState::Ok {
            next_turn: floor.get_current_turn() + 1,
        };

        update = update.log(FloorEvent::Move(MoveEvent {
            subject: subject_clone.id,
            tile: subject_clone.pos,
        }));

        update.bind(|floor| floor.update_entity(Rc::new(subject_clone)))
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

        if !floor
            .occupiers
            .contains_key(&(floor.entities[subject_id].pos + dir))
        {
            return None;
        }

        Some(Box::new(BumpCommand { dir, subject_id }))
    }
}

#[derive(Debug)]
struct BumpCommand {
    dir: RelativePosition,
    subject_id: EntityId,
}

impl CommandTrait for BumpCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut update = BorrowedFloorUpdate::new(floor);

        let mut subject_clone: Entity = (*floor.entities[self.subject_id]).clone();
        subject_clone.state = EntityState::Ok {
            next_turn: floor.get_current_turn() + 1,
        };

        update = update.log(FloorEvent::StartAttack(StartAttackEvent {
            subject: subject_clone.id,
            tile: subject_clone.pos + self.dir,
        }));

        let object_index = floor.occupiers[&(subject_clone.pos + self.dir)];

        let object_ref = &floor.entities[object_index];
        let mut object_clone: Entity = (**object_ref).clone();
        object_clone.health -= 1;

        update = update.log(FloorEvent::AttackHit(AttackHitEvent {
            subject: subject_clone.id,
            target: object_clone.id,
            damage: 1,
        }));

        let update = update.bind(|floor| {
            floor.update_entities(Vec::from([Rc::new(subject_clone), Rc::new(object_clone)]))
        });

        update.bind(|floor| {
            TakeKnockbackUtil {
                entity: object_index,
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
        let step_action = StepAction;
        let verify_action = step_action.verify_action(
            floor,
            self.subject_id,
            // TODO: Read from pathfinding.
            RelativePosition {
                dx: (self.tile.x - subject_pos.x).clamp(-1, 1),
                dy: (self.tile.y - subject_pos.y).clamp(-1, 1),
            },
        );

        // HACK: this should go to EntityState::OK.extra_actions or something idk

        match verify_action {
            None => {
                // Give up immediately.
                // Even when pathfinding is implemented, a failed step should probably mean to stop.
                let mut subject_clone: Entity = (*floor.entities[self.subject_id]).clone();
                subject_clone.state = EntityState::Ok {
                    next_turn: floor.get_current_turn(),
                };
                floor.update_entity(Rc::new(subject_clone))
            }
            Some(command) => {
                let update = command.do_action(floor);

                update.bind(|floor| {
                    let mut subject_clone: Entity = (*floor.entities[self.subject_id]).clone();
                    subject_clone.state = if floor.entities[self.subject_id].pos == self.tile {
                        EntityState::Ok {
                            next_turn: floor.get_current_turn(),
                        }
                    } else {
                        EntityState::ConfirmCommand {
                            next_turn: floor.get_current_turn(),
                            to_confirm: Rc::new(self.clone()),
                        }
                    };
                    floor.update_entity(Rc::new(subject_clone))
                })
            }
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

#[cfg(test)]
#[test]
fn bump_test() {
    use crate::{
        entity::{EntityId, EntityState},
        positional::AbsolutePosition,
    };

    let mut update = FloorUpdate::new(Floor::new());
    let player_id;
    let other_id;
    (update, player_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            id: EntityId::default(),
            state: EntityState::Ok { next_turn: 0 },
            pos: AbsolutePosition::new(0, 0),
            health: 0,
            is_player_controlled: false,
        })
    });
    (update, other_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            id: EntityId::default(),
            state: EntityState::Ok { next_turn: 0 },
            pos: AbsolutePosition::new(1, 0),
            health: 0,
            is_player_controlled: false,
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
    use crate::{
        entity::{EntityId, EntityState},
        positional::AbsolutePosition,
    };

    let mut update = FloorUpdate::new(Floor::new());
    let player_id;
    (update, player_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            id: EntityId::default(),
            state: EntityState::Ok { next_turn: 0 },
            pos: AbsolutePosition::new(0, 0),
            health: 0,
            is_player_controlled: true,
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

    update = update.bind(confirm_command);
    update = update.bind(confirm_command);
    update = update.bind(confirm_command);
    update = update.bind(confirm_command);

    let (floor, _) = update.into_both();
    assert!(matches!(
        floor.entities[player_id].state,
        EntityState::Ok { next_turn: 5 }
    ));
    assert_eq!(floor.entities[player_id].pos, AbsolutePosition::new(5, 3));
}
