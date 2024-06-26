use std::rc::Rc;

use crate::actions::DeserializeActionTrait;
use crate::actions::DeserializeCommandTrait;
use crate::actions::SerializeActionTrait;
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
use crate::positional::RelativePosition;

use super::events::AttackHitEvent;
use super::events::StartAttackEvent;
use super::ActionTrait;
use super::CommandTrait;
use super::DirectionActionTrait;
use super::FloorEvent;
use super::SerializableAction;

// Hits once, then queues another.
#[derive(Debug)]
pub struct DoubleHitAction;

impl DirectionActionTrait for DoubleHitAction {
    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
        dir: RelativePosition,
    ) -> Option<Box<dyn CommandTrait>> {
        assert!(floor.entities.contains(subject_ref));

        if dir.length() > 1 {
            return None;
        }

        Some(Box::new(DoubleHitCommand {
            dir,
            subject_ref: Rc::clone(subject_ref),
        }))
    }
}

#[derive(Debug)]
struct DoubleHitCommand {
    dir: RelativePosition,
    subject_ref: Rc<Entity>,
}

impl CommandTrait for DoubleHitCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut update = BorrowedFloorUpdate::new(floor);

        let mut dirty = Vec::new();

        let mut subject_clone: Entity = (*self.subject_ref).clone();
        subject_clone.state = EntityState::Committed {
            next_turn: floor.get_current_turn() + 1,
            queued_command: Rc::new(DoubleHitFollowup {
                dir: self.dir,
                subject_id: subject_clone.id,
            }),
        };

        update = update.log(FloorEvent::StartAttack(StartAttackEvent {
            subject: subject_clone.id,
            tile: self.subject_ref.pos + self.dir,
        }));

        if let Some(&object_index) = floor.occupiers.get(&(self.subject_ref.pos + self.dir)) {
            let object_ref = &floor.entities[object_index];
            let mut object_clone: Entity = (**object_ref).clone();
            object_clone.health -= 1;

            update = update.log(FloorEvent::AttackHit(AttackHitEvent {
                subject: subject_clone.id,
                target: object_clone.id,
                damage: 1,
            }));

            dirty.push(Rc::new(object_clone));
        } else {
            println!("Hit no one.");
        }

        dirty.push(Rc::new(subject_clone));
        update.bind(|floor| floor.update_entities(dirty))
    }
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug, TypeName))]
struct DoubleHitFollowup {
    dir: RelativePosition,
    // TODO: Either restore Rc<Entity>, or decide if EntityId everywhere else is preferable.
    subject_id: EntityId,
}

#[archive_dyn(deserialize)]
impl CommandTrait for DoubleHitFollowup {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut update = BorrowedFloorUpdate::new(floor);

        let mut dirty = Vec::new();

        let mut subject_clone: Entity = (*floor.entities[self.subject_id]).clone();
        subject_clone.state = EntityState::Ok {
            next_turn: floor.get_current_turn() + 1,
        };

        if let Some(&object_index) = floor.occupiers.get(&(subject_clone.pos + self.dir)) {
            let object_ref = &floor.entities[object_index];
            let mut object_clone: Entity = (**object_ref).clone();
            object_clone.health -= 1;

            update = update.log(FloorEvent::AttackHit(AttackHitEvent {
                subject: subject_clone.id,
                target: object_clone.id,
                damage: 1,
            }));

            dirty.push(Rc::new(object_clone));
        } else {
            println!("Hit no one.");
        }

        dirty.push(Rc::new(subject_clone));
        update.bind(|floor| floor.update_entities(dirty))
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
    fn verify_action(&self, _: &Floor, subject_ref: &Rc<Entity>) -> Option<Box<dyn CommandTrait>> {
        Some(Box::new(EnterStanceCommand {
            subject_ref: Rc::clone(subject_ref),
        }))
    }
}

#[derive(Debug)]
pub struct EnterStanceCommand {
    subject_ref: Rc<Entity>,
}

impl CommandTrait for EnterStanceCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = (*self.subject_ref).clone();
        subject_clone.state = EntityState::RestrictedActions {
            next_turn: floor.get_current_turn() + 1,
            restricted_actions: vec![SerializableAction::None(Rc::new(ExitStanceAction))],
        };

        floor.update_entity(Rc::new(subject_clone))
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug, TypeName))]
pub struct ExitStanceAction;

#[archive_dyn(deserialize)]
impl ActionTrait for ExitStanceAction {
    fn verify_action(&self, _: &Floor, subject_ref: &Rc<Entity>) -> Option<Box<dyn CommandTrait>> {
        Some(Box::new(ExitStanceCommand {
            subject_ref: Rc::clone(subject_ref),
        }))
    }
}

impl ActionTrait for Archived<ExitStanceAction> {
    fn verify_action(&self, _: &Floor, subject_ref: &Rc<Entity>) -> Option<Box<dyn CommandTrait>> {
        Some(Box::new(ExitStanceCommand {
            subject_ref: Rc::clone(subject_ref),
        }))
    }
}

#[derive(Debug)]
pub struct ExitStanceCommand {
    subject_ref: Rc<Entity>,
}

impl CommandTrait for ExitStanceCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = (*self.subject_ref).clone();
        subject_clone.state = EntityState::Ok {
            next_turn: floor.get_current_turn(),
        };

        floor.update_entity(Rc::new(subject_clone))
    }
}

#[cfg(test)]
#[test]
fn double_hit() {
    use crate::{
        entity::{EntityId, EntityState},
        floor::TurntakingError,
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
            is_player_controlled: true,
        })
    });
    (update, other_id) = update.bind_with_side_output(|floor| {
        floor.add_entity(Entity {
            id: EntityId::default(),
            state: EntityState::Ok { next_turn: 100 },
            pos: AbsolutePosition::new(1, 0),
            health: 0,
            is_player_controlled: true,
        })
    });
    update = update.bind(|floor| {
        DoubleHitAction
            .verify_action(
                &floor,
                &floor.entities[player_id],
                RelativePosition::new(1, 0),
            )
            .unwrap()
            .do_action(&floor)
    });
    update = update.bind(|floor| floor.take_npc_turn().unwrap()); // Second hit.

    assert_eq!(
        update.get_contents().take_npc_turn().err(),
        Some(TurntakingError::PlayerTurn { who: player_id })
    );

    let (_, log) = update.into_both();
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
            FloorEvent::AttackHit(AttackHitEvent {
                subject: player_id,
                target: other_id,
                damage: 1,
            }),
        ]
    );
}
