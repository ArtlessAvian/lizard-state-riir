use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Entity {
    pub x: i8,
}

#[derive(Clone)]
pub struct Floor {
    pub entities: Vec<Rc<Entity>>,
    pub occupiers: HashMap<i8, Weak<Entity>>,
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
        clone.entities.push(new.clone());
        match clone.occupiers.entry(new.x) {
            std::collections::hash_map::Entry::Occupied(_) => panic!("AHHHHHHH"),
            std::collections::hash_map::Entry::Vacant(vacancy) => {
                vacancy.insert(Rc::downgrade(&new));
            }
        }
        clone
    }

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

        let mut new_occupiers = self.occupiers.clone();
        new_occupiers.remove_entry(&old.x);
        match new_occupiers.entry(new.x) {
            std::collections::hash_map::Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
            std::collections::hash_map::Entry::Vacant(vacancy) => {
                vacancy.insert(Rc::downgrade(&new));
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
            .map(|x| map.get(x).unwrap_or(x).clone())
            .collect::<Vec<Rc<Entity>>>();

        let mut new_occupiers = self.occupiers.clone();
        for old in map.keys() {
            new_occupiers.remove_entry(&old.x);
        }
        for new in map.values() {
            match new_occupiers.entry(new.x) {
                std::collections::hash_map::Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
                std::collections::hash_map::Entry::Vacant(vacancy) => {
                    vacancy.insert(Rc::downgrade(new));
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
    fn verify_action(&self, floor: &Floor, subject_ref: &Rc<Entity>) -> Option<impl CommandTrait>;
}

pub trait CommandTrait {
    fn do_action(self, floor: &Floor) -> Floor;
}
