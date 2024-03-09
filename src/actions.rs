use std::collections::HashMap;
use std::rc::Rc;

use crate::data::ActionTrait;
use crate::data::CommandTrait;
use crate::data::Entity;
use crate::data::Floor;
use crate::data::FloorSnapshot;

pub struct DoNothingAction;
impl ActionTrait for DoNothingAction {
    fn verify_action(&self, floor: &Floor, e: &Rc<Entity>) -> Option<Self> {
        assert!(floor.entities.contains(e));
        Some(DoNothingAction)
    }
}
impl CommandTrait for DoNothingAction {
    fn do_action(self, floor: &mut Floor) -> FloorSnapshot {
        floor.get_snapshot()
    }
}

pub struct GoRightAction;
impl ActionTrait for GoRightAction {
    fn verify_action(&self, floor: &Floor, subject_ref: &Rc<Entity>) -> Option<GoRightCommand> {
        assert!(floor.entities.contains(subject_ref));
        if floor.entities.iter().all(|e| e.x != subject_ref.x + 1) {
            Some(GoRightCommand {
                subject_ref: subject_ref.clone(),
            })
        } else {
            None
        }
    }
}

pub struct GoRightCommand {
    subject_ref: Rc<Entity>,
}
impl CommandTrait for GoRightCommand {
    // TODO: assumes entity is on floor
    fn do_action(self, floor: &mut Floor) -> FloorSnapshot {
        let mut subject_clone: Entity = (*self.subject_ref).clone();

        subject_clone.x += 1;

        floor.update_entity(self.subject_ref, Rc::new(subject_clone))
    }
}

pub struct EveryoneGoRightAction;
impl ActionTrait for EveryoneGoRightAction {
    fn verify_action(&self, floor: &Floor, e: &Rc<Entity>) -> Option<EveryoneGoRightCommand> {
        assert!(floor.entities.contains(e));
        Some(EveryoneGoRightCommand)
    }
}

pub struct EveryoneGoRightCommand;
impl CommandTrait for EveryoneGoRightCommand {
    // TODO: assumes entity is on floor
    fn do_action(self, floor: &mut Floor) -> FloorSnapshot {
        let mut map = HashMap::new();
        for entity in &floor.entities {
            let mut clone = entity.as_ref().clone();
            clone.x += 1;
            map.insert(entity.clone(), Rc::new(clone));
        }
        floor.update_entities(map)
    }
}

pub struct AttackRightAction;
impl ActionTrait for AttackRightAction {
    fn verify_action(&self, floor: &Floor, e: &Rc<Entity>) -> Option<AttackRightCommand> {
        assert!(floor.entities.contains(e));
        let target = floor.entities.iter().find(|other| other.x == e.x + 1)?;
        Some(AttackRightCommand {
            subject_ref: Rc::clone(e),
            target_ref: Rc::clone(target),
        })
    }
}

pub struct AttackRightCommand {
    subject_ref: Rc<Entity>,
    target_ref: Rc<Entity>,
}
impl CommandTrait for AttackRightCommand {
    fn do_action(self, floor: &mut Floor) -> FloorSnapshot {
        println!(
            "subject at {} hits target at {}",
            self.subject_ref.x, self.target_ref.x
        );
        floor.get_snapshot()
    }
}
