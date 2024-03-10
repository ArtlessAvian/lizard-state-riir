use serde::Deserialize;
use serde::Serialize;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use crate::actions::GoRightAction;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub x: i8,
}

impl Entity {
    pub fn get_actions() -> Box<dyn ActionTrait> {
        Box::new(GoRightAction)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Floor {
    // Rc is shared between Floor generations.
    // Prefer to use indices since serializing Rcs does not preserve identity.
    pub entities: Vec<Rc<Entity>>,
    pub occupiers: HashMap<i8, Rc<Entity>>,
}

impl Floor {
    pub fn new() -> Self {
        Floor {
            entities: Vec::new(),
            occupiers: HashMap::new(),
        }
    }

    pub fn add_entity(&self, new: Rc<Entity>) -> Self {
        let mut clone = self.clone();

        clone.entities.push(Rc::clone(&new));

        match clone.occupiers.entry(new.x) {
            Entry::Occupied(_) => panic!("AHHHHHHH"),
            Entry::Vacant(vacancy) => {
                vacancy.insert(Rc::clone(&new));
            }
        }

        clone
    }

    pub fn get_player(&self) -> Rc<Entity> {
        Rc::clone(self.entities.first().unwrap())
    }

    pub fn get_someone(&self) -> Rc<Entity> {
        Rc::clone(self.entities.last().unwrap())
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

        let mut new_occupiers = self.occupiers.clone();
        new_occupiers.remove_entry(&old.x);
        match new_occupiers.entry(new.x) {
            Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
            Entry::Vacant(vacancy) => {
                vacancy.insert(Rc::clone(&new));
            }
        };

        Floor {
            entities: new_entities,
            occupiers: new_occupiers,
        }
    }

    pub fn update_entities(&self, map: HashMap<Rc<Entity>, Rc<Entity>>) -> Floor {
        let new_entities = self
            .entities
            .iter()
            .map(|x| Rc::clone(map.get(x).unwrap_or(x)))
            .collect::<Vec<Rc<Entity>>>();

        let mut new_occupiers = self.occupiers.clone();
        for old in map.keys() {
            new_occupiers.remove_entry(&old.x);
        }
        for new in map.values() {
            match new_occupiers.entry(new.x) {
                Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
                Entry::Vacant(vacancy) => {
                    vacancy.insert(Rc::clone(new));
                }
            };
        }

        Floor {
            entities: new_entities,
            occupiers: new_occupiers,
        }
    }
}

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
