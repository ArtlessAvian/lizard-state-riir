pub mod example;

use std::rc::Rc;

use crate::data::Entity;
use crate::data::Floor;

pub trait ActionTrait {
    // not object safe
    // fn verify_action(&self, floor: &Floor, subject_ref: &Rc<Entity>) -> Option<impl CommandTrait>;

    fn verify_action(
        &self,
        floor: &Floor,
        subject_ref: &Rc<Entity>,
    ) -> Option<Box<dyn CommandTrait>>;
}

pub trait CommandTrait {
    fn do_action(&self, floor: &Floor) -> Floor;
}
