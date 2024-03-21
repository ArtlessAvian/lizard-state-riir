use serde::Deserialize;
use serde::Serialize;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use crate::actions::ActionTrait;
use crate::actions::NullAction;
use crate::positional::AbsolutePosition;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub pos: AbsolutePosition,
}

impl Entity {
    pub fn get_actions() -> Box<dyn ActionTrait> {
        Box::new(NullAction {})
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Floor {
    // Rc is shared between Floor generations.
    // Prefer to use indices since serializing Rcs does not preserve identity.
    pub entities: Vec<Rc<Entity>>,
    pub occupiers: HashMap<AbsolutePosition, usize>,
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

        let index = clone.entities.len();
        clone.entities.push(Rc::clone(&new));

        match clone.occupiers.entry(new.pos) {
            Entry::Occupied(_) => panic!("AHHHHHHH"),
            Entry::Vacant(vacancy) => {
                vacancy.insert(index);
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
        let mut new_entities = self.entities.clone();

        let (index, thing) = new_entities
            .iter_mut()
            .enumerate()
            .find(|(_i, x)| Rc::ptr_eq(x, &old))
            .unwrap();

        *thing = new.clone();

        let mut new_occupiers = self.occupiers.clone();
        new_occupiers.remove_entry(&old.pos);
        match new_occupiers.entry(new.pos) {
            Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
            Entry::Vacant(vacancy) => {
                vacancy.insert(index);
            }
        };

        Floor {
            entities: new_entities,
            occupiers: new_occupiers,
        }
    }

    pub fn update_entities(&self, map: HashMap<Rc<Entity>, Rc<Entity>>) -> Floor {
        let index_map = self
            .entities
            .iter()
            .enumerate()
            .filter(|(_i, x)| map.contains_key(*x))
            .map(|(i, x)| (x.clone(), i))
            .collect::<HashMap<Rc<Entity>, usize>>();

        let new_entities = self
            .entities
            .iter()
            .map(|x| Rc::clone(map.get(x).unwrap_or(x)))
            .collect();

        let mut new_occupiers = self.occupiers.clone();
        for old in map.keys() {
            new_occupiers.remove(&old.pos);
        }
        for old in map.keys() {
            match new_occupiers.entry(map[old].pos) {
                Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
                Entry::Vacant(vacancy) => {
                    vacancy.insert(index_map[old]);
                }
            }
        }

        Floor {
            entities: new_entities,
            occupiers: new_occupiers,
        }
    }
}

impl Default for Floor {
    fn default() -> Self {
        Floor::new()
    }
}
