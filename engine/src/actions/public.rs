use std::rc::Rc;

use crate::data::Entity;
use crate::data::Floor;
use crate::positional::RelativePosition;

use super::ActionTrait;
use super::CommandTrait;

/// Publicly exposed actions, free to construct.
/// Not hidden behind entities.

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
        floor.update_entity(Rc::clone(&self.subject_ref), Rc::new(subject_clone))
    }
}
