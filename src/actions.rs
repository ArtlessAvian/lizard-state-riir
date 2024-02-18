use std::{process::Command, rc::Rc};

use crate::data::{ActionTrait, CommandTrait, Entity, Floor};

pub struct DoNothingAction;
impl ActionTrait for DoNothingAction {
    fn verify_action(&self, floor: &Floor, e: Rc<Entity>) -> Option<Self> {
        Some(DoNothingAction)
    }
}
impl CommandTrait for DoNothingAction {
    fn do_action(self, _floor: &mut Floor) {}
}

pub struct GoRightAction;
impl ActionTrait for GoRightAction {
    fn verify_action(&self, floor: &Floor, subject_ref: Rc<Entity>) -> Option<GoRightCommand> {
        if floor.entities.iter().all(|e| e.x != subject_ref.x + 1) {
            Some(GoRightCommand {
                subject_ref,
                nobody_right_of_subject: (),
            })
        } else {
            None
        }
    }
}

pub struct GoRightCommand {
    subject_ref: Rc<Entity>,
    nobody_right_of_subject: (),
}
impl CommandTrait for GoRightCommand {
    // TODO: assumes entity is on floor
    fn do_action(self, floor: &mut Floor) {
        let mut subject = floor
            .entities
            .iter_mut()
            .find(|e| Rc::ptr_eq(&e, &self.subject_ref))
            .unwrap();

        drop(self.subject_ref);

        Rc::get_mut(&mut subject).unwrap().x += 1;
    }
}

pub struct EveryoneGoRightAction;
impl ActionTrait for EveryoneGoRightAction {
    fn verify_action(&self, floor: &Floor, e: Rc<Entity>) -> Option<EveryoneGoRightCommand> {
        Some(EveryoneGoRightCommand)
    }
}

pub struct EveryoneGoRightCommand;
impl CommandTrait for EveryoneGoRightCommand {
    // TODO: assumes entity is on floor
    fn do_action(self, floor: &mut Floor) {
        floor.entities.iter_mut().for_each(|mut x| {
            Rc::get_mut(&mut x).unwrap().x += 1;
        });
    }
}
