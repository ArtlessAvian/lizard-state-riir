use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash)]
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

    pub fn update_entity(&self, old: Rc<Entity>, new: Rc<Entity>) -> Floor {
        let new_entities = self
            .entities
            .iter()
            .map(|x| {
                if Rc::ptr_eq(x, &old) {
                    new.clone()
                } else {
                    x.clone()
                }
            })
            .collect::<Vec<Rc<Entity>>>();
        Floor {
            entities: new_entities,
        }
    }

    pub fn update_entities(&self, map: HashMap<Rc<Entity>, Rc<Entity>>) -> Floor {
        let new_entities = self
            .entities
            .iter()
            .map(|x| map.get(x).unwrap_or(x).clone())
            .collect::<Vec<Rc<Entity>>>();
        Floor {
            entities: new_entities,
        }
    }
}

impl Clone for Floor {
    fn clone(&self) -> Self {
        Self {
            entities: self.entities.clone(),
        }
    }
}

pub trait ActionTrait {
    fn verify_action(&self, floor: &Floor, subject_ref: Rc<Entity>) -> Option<impl CommandTrait>;
}

pub trait CommandTrait {
    fn do_action(self, floor: &Floor) -> Floor;
}
