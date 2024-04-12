use std::collections::HashSet;
use std::rc::Rc;

use crate::data::Floor;
use crate::entity::Entity;
use crate::positional::RelativePosition;

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

struct StepCommand {
    dir: RelativePosition,
    subject_ref: Rc<Entity>,
}

impl CommandTrait for StepCommand {
    fn do_action(&self, floor: &Floor) -> (Floor, Vec<FloorEvent>) {
        let mut log = Vec::new();

        let mut subject_clone: Entity = (*self.subject_ref).clone();
        subject_clone.pos = subject_clone.pos + self.dir;
        *subject_clone.next_turn.as_mut().unwrap() += 1;

        log.push(FloorEvent::Move(subject_clone.id, subject_clone.pos));

        (floor.update_entity(Rc::new(subject_clone)), log)
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

struct BumpCommand {
    dir: RelativePosition,
    subject_ref: Rc<Entity>,
}

impl CommandTrait for BumpCommand {
    fn do_action(&self, floor: &Floor) -> (Floor, Vec<FloorEvent>) {
        let mut log = Vec::new();

        let mut subject_clone: Entity = (*self.subject_ref).clone();
        *subject_clone.next_turn.as_mut().unwrap() += 1;

        log.push(FloorEvent::StartAttack(subject_clone.id, self.dir));

        let object_index = floor.occupiers[&(self.subject_ref.pos + self.dir)];

        let object_ref = &floor.entities[object_index];
        let mut object_clone: Entity = (**object_ref).clone();
        object_clone.health -= 1;

        log.push(FloorEvent::AttackHit(subject_clone.id, object_clone.id, 1));

        (
            floor.update_entities(HashSet::from([
                Rc::new(subject_clone),
                Rc::new(object_clone),
            ])),
            log,
        )
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
    use crate::{entity::EntityId, positional::AbsolutePosition};

    let mut floor = Floor::new();
    let player_id;
    (floor, player_id) = floor.add_entity(Entity {
        id: EntityId::default(),
        next_turn: Some(0),
        pos: AbsolutePosition::new(0, 0),
        health: 0,
    });
    let other_id;
    (floor, other_id) = floor.add_entity(Entity {
        id: EntityId::default(),
        next_turn: Some(0),
        pos: AbsolutePosition::new(1, 0),
        health: 0,
    });
    let log;
    (floor, log) = BumpAction {
        dir: RelativePosition::new(1, 0),
    }
    .verify_action(&floor, &floor.entities[player_id])
    .unwrap()
    .do_action(&floor);
    dbg!(floor);

    assert_eq!(
        log,
        vec![
            FloorEvent::StartAttack(player_id, RelativePosition::new(1, 0)),
            FloorEvent::AttackHit(player_id, other_id, 1)
        ]
    );
}
