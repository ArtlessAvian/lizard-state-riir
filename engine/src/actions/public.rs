use std::collections::HashSet;
use std::rc::Rc;

use crate::data::Entity;
use crate::data::Floor;
use crate::positional::RelativePosition;

use super::ActionTrait;
use super::CommandTrait;

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
    fn do_action(&self, floor: &Floor) -> Floor {
        let mut subject_clone: Entity = (*self.subject_ref).clone();
        subject_clone.pos = subject_clone.pos + self.dir;
        *subject_clone.next_turn.as_mut().unwrap() += 1;
        floor.update_entity(Rc::new(subject_clone))
    }
}

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
    fn do_action(&self, floor: &Floor) -> Floor {
        let mut subject_clone: Entity = (*self.subject_ref).clone();
        *subject_clone.next_turn.as_mut().unwrap() += 1;

        let object_index = floor.occupiers[&(self.subject_ref.pos + self.dir)];

        let object_ref = &floor.entities[object_index];
        let mut object_clone: Entity = (**object_ref).clone();
        object_clone.health -= 1;

        floor.update_entities(HashSet::from([
            Rc::new(subject_clone),
            Rc::new(object_clone),
        ]))
    }
}

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
    use crate::positional::AbsolutePosition;

    let mut floor = Floor::new();
    floor = floor.add_entity(Entity {
        id: 0,
        next_turn: Some(0),
        pos: AbsolutePosition { x: 0, y: 0 },
        health: 0,
    });
    floor = floor.add_entity(Entity {
        id: 1,
        next_turn: Some(0),
        pos: AbsolutePosition { x: 1, y: 0 },
        health: 0,
    });
    floor = BumpAction {
        dir: RelativePosition { dx: 1, dy: 0 },
    }
    .verify_action(&floor, &floor.get_player())
    .unwrap()
    .do_action(&floor);
    dbg!(floor);
}
