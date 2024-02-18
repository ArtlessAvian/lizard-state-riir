use std::rc::Rc;

pub struct Entity {
    pub x: i8,
}

pub struct Floor {
    pub entities: Vec<Rc<Entity>>,
}

impl Floor {
    pub fn get_player(&self) -> Rc<Entity> {
        self.entities[0].clone()
    }

    pub fn get_someone(&self) -> Rc<Entity> {
        self.entities.last().unwrap().clone()
    }
}

pub trait ActionTrait {
    fn verify_action(&self, floor: &Floor, subject_ref: Rc<Entity>) -> Option<impl CommandTrait>;
}

pub trait CommandTrait {
    fn do_action(self, floor: &mut Floor);
}
