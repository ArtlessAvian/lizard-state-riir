use std::rc::Rc;

use crate::data::BorrowedFloorUpdate;
use crate::data::Floor;
use crate::data::FloorUpdate;
use crate::entity::Entity;
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

        if self.dir.length() > 1 {
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

#[cfg(test)]
#[test]
fn bump_test() {
    use crate::{
        entity::{EntityId, EntityState},
        positional::AbsolutePosition,
    };

    let update = FloorUpdate::new(Floor::new());
    let (update, player_id) = update.bind_with_side_output(|floor| {
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
    let (update, other_id) = update.bind_with_side_output(|floor| {
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

    // discard the log.
    let mut update = FloorUpdate::new(update.get_contents().clone());
    update = update.bind(|floor| {
        BumpAction {
            dir: RelativePosition::new(1, 0),
        }
        .verify_action(floor, &floor.entities[player_id])
        .unwrap()
        .do_action(floor)
    });

    let (floor, log) = update.into_both();

    dbg!(floor);
    assert_eq!(
        log,
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
