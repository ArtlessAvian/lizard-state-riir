use serde::Deserialize;
use serde::Serialize;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::actions::ActionTrait;
use crate::actions::NullAction;
use crate::positional::AbsolutePosition;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub id: usize,

    pub pos: AbsolutePosition,
    pub health: i8,
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

    pub fn add_entity(&self, mut new: Entity) -> Self {
        new.id = self.entities.len();
        let id = new.id;

        let mut next_entities = self.entities.clone();
        next_entities.push(Rc::new(new));

        let mut next_occupiers = self.occupiers.clone();
        match next_occupiers.entry(next_entities[id].pos) {
            Entry::Occupied(_) => panic!("AHHHHHHH"),
            Entry::Vacant(vacancy) => {
                vacancy.insert(id);
            }
        }

        Floor {
            entities: next_entities,
            occupiers: next_occupiers,
        }
    }

    pub fn get_player(&self) -> Rc<Entity> {
        Rc::clone(self.entities.first().unwrap())
    }

    pub fn get_someone(&self) -> Rc<Entity> {
        Rc::clone(self.entities.last().unwrap())
    }

    pub fn update_entity(&self, new: Rc<Entity>) -> Floor {
        let old = &self.entities[new.id];

        let mut next_entities = self.entities.clone();
        next_entities[new.id] = Rc::clone(&new);

        let mut next_occupiers = self.occupiers.clone();
        next_occupiers.remove_entry(&old.pos);
        match next_occupiers.entry(new.pos) {
            Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
            Entry::Vacant(vacancy) => {
                vacancy.insert(new.id);
            }
        };

        Floor {
            entities: next_entities,
            occupiers: next_occupiers,
        }
    }

    pub fn update_entities(&self, new_set: HashSet<Rc<Entity>>) -> Floor {
        let old_set = new_set
            .iter()
            .map(|x| &self.entities[x.id])
            .collect::<HashSet<&Rc<Entity>>>();

        let mut next_entities = self.entities.clone();
        for new in &new_set {
            next_entities[new.id] = Rc::clone(new);
        }

        let mut next_occupiers = self.occupiers.clone();
        for old in &old_set {
            next_occupiers.remove(&old.pos);
        }
        for new in &new_set {
            match next_occupiers.entry(new.pos) {
                Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
                Entry::Vacant(vacancy) => {
                    vacancy.insert(new.id);
                }
            }
        }

        Floor {
            entities: next_entities,
            occupiers: next_occupiers,
        }
    }
}

impl Default for Floor {
    fn default() -> Self {
        Floor::new()
    }
}
