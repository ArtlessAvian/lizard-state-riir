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
use super::ActionTrait;
use super::CommandTrait;
use super::FloorEvent;

/// Moves one space.
pub struct StepAction {
    pub dir: RelativePosition,
}

impl ActionTrait for StepAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
    ) -> Option<Box<dyn CommandTrait>> {
        assert!(floor.entities.contains(subject_ref));

        if self.dir.length() > 1 {
            return None;
        }

        if !floor.map.is_tile_floor(&(subject_ref.pos + self.dir)) {
            return None;
        }

        // TODO: Decide whether to make impossible (leave code as is)
        // or to waste a turn (check in command, no-op if occupied.)
        if floor
            .occupiers
            .get(&(subject_ref.pos + self.dir))
            .is_some_and(|x| x != &subject_ref.id)
        {
            return None;
        }

        Some(Box::new(StepCommand {
            dir: self.dir,
            subject_ref: Rc::clone(subject_ref),
        }))
    }
}

#[derive(Debug)]
struct StepCommand {
    dir: RelativePosition,
    subject_ref: Rc<Entity>,
}

impl CommandTrait for StepCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut update = BorrowedFloorUpdate::new(floor);

        let mut subject_clone: Entity = (*self.subject_ref).clone();
        subject_clone.pos = subject_clone.pos + self.dir;
        *subject_clone.next_turn.as_mut().unwrap() += 1;

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
pub struct BumpAction {
    pub dir: RelativePosition,
}

impl ActionTrait for BumpAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
    ) -> Option<Box<dyn CommandTrait>> {
        assert!(floor.entities.contains(subject_ref));

        if self.dir.length() != 1 {
            return None;
        }

        if !floor.occupiers.contains_key(&(subject_ref.pos + self.dir)) {
            return None;
        }

        Some(Box::new(BumpCommand {
            dir: self.dir,
            subject_ref: Rc::clone(subject_ref),
        }))
    }
}

#[derive(Debug)]
struct BumpCommand {
    dir: RelativePosition,
    subject_ref: Rc<Entity>,
}

impl CommandTrait for BumpCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut update = BorrowedFloorUpdate::new(floor);

        let mut subject_clone: Entity = (*self.subject_ref).clone();
        *subject_clone.next_turn.as_mut().unwrap() += 1;

        update = update.log(FloorEvent::StartAttack(StartAttackEvent {
            subject: subject_clone.id,
            tile: self.subject_ref.pos + self.dir,
        }));

        let object_index = floor.occupiers[&(self.subject_ref.pos + self.dir)];

        let object_ref = &floor.entities[object_index];
        let mut object_clone: Entity = (**object_ref).clone();
        object_clone.health -= 1;

        update = update.log(FloorEvent::AttackHit(AttackHitEvent {
            subject: subject_clone.id,
            target: object_clone.id,
            damage: 1,
        }));

        update.bind(|floor| {
            floor.update_entities(Vec::from([Rc::new(subject_clone), Rc::new(object_clone)]))
        })
    }
}

/// In order, tries to Bump, Walk, or no-op.
///
/// TODO: Maybe move to a submodule.
pub struct StepMacroAction {
    pub dir: RelativePosition,
}

impl ActionTrait for StepMacroAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
    ) -> Option<Box<dyn CommandTrait>> {
        let bump = BumpAction { dir: self.dir };
        if let Some(command) = bump.verify_action(floor, subject_ref) {
            return Some(command);
        }

        let step = StepAction { dir: self.dir };
        if let Some(command) = step.verify_action(floor, subject_ref) {
            return Some(command);
        }

        None
    }
}

#[derive(Debug)]
pub struct GotoAction {
    pub tile: AbsolutePosition,
}

impl ActionTrait for GotoAction {
    fn verify_action(
        &self,
        _floor: &Floor,
        subject_ref: &Rc<Entity>,
    ) -> Option<Box<dyn CommandTrait>> {
        // Pathfind to target.
        Some(Box::new(GotoCommand {
            tile: self.tile,
            subject_id: subject_ref.id,
        }))
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
        let step_action = StepAction {
            // TODO: Read from pathfinding.
            dir: RelativePosition {
                dx: (self.tile.x - subject_pos.x).clamp(-1, 1),
                dy: (self.tile.y - subject_pos.y).clamp(-1, 1),
            },
        };
        let verify_action = step_action.verify_action(floor, &floor.entities[self.subject_id]);

        // HACK: this should go to EntityState::OK.extra_actions or something idk

        match verify_action {
            None => {
                // Give up immediately.
                // Even when pathfinding is implemented, a failed step should probably mean to stop.
                let mut subject_clone: Entity = (*floor.entities[self.subject_id]).clone();
                subject_clone.state = EntityState::Ok {
                    queued_command: None,
                };
                floor.update_entity(Rc::new(subject_clone))
            }
            Some(command) => {
                let update = command.do_action(floor);

                update.bind(|floor| {
                    if floor.entities[self.subject_id].pos != self.tile {
                        let mut subject_clone: Entity = (*floor.entities[self.subject_id]).clone();
                        subject_clone.state = EntityState::Ok {
                            queued_command: Some(Rc::new(self.clone())),
                        };
                        floor.update_entity(Rc::new(subject_clone))
                    } else {
                        let mut subject_clone: Entity = (*floor.entities[self.subject_id]).clone();
                        subject_clone.state = EntityState::Ok {
                            queued_command: None,
                        };
                        floor.update_entity(Rc::new(subject_clone))
                    }
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
            next_turn: Some(0),
            state: EntityState::Ok {
                queued_command: None,
            },
            pos: AbsolutePosition::new(0, 0),
            health: 0,
        })
    });
    (update, other_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            id: EntityId::default(),
            next_turn: Some(0),
            state: EntityState::Ok {
                queued_command: None,
            },
            pos: AbsolutePosition::new(1, 0),
            health: 0,
        })
    });
    update = update.bind(|floor| {
        BumpAction {
            dir: RelativePosition::new(1, 0),
        }
        .verify_action(&floor, &floor.entities[player_id])
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
            next_turn: Some(0),
            state: EntityState::Ok {
                queued_command: None,
            },
            pos: AbsolutePosition::new(0, 0),
            health: 0,
        })
    });
    update = update.bind(|floor| {
        GotoAction {
            tile: AbsolutePosition::new(5, 3),
        }
        .verify_action(&floor, &floor.entities[player_id])
        .unwrap()
        .do_action(&floor)
    });
    update = update.bind(|floor| floor.take_npc_turn().unwrap());
    update = update.bind(|floor| floor.take_npc_turn().unwrap());
    update = update.bind(|floor| floor.take_npc_turn().unwrap());
    update = update.bind(|floor| floor.take_npc_turn().unwrap());

    let (floor, log) = update.into_both();
    floor.take_npc_turn().unwrap_err();
    assert_eq!(floor.entities[player_id].pos, AbsolutePosition::new(5, 3))
}
