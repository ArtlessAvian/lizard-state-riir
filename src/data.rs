use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Entity {
    pub x: i8,
}

#[derive(Clone)]
pub struct Floor {
    pub entities: Vec<Rc<Entity>>,
    // Invariants
    pub occupiers: HashMap<i8, Rc<Entity>>,
}

#[non_exhaustive]
pub struct FloorSnapshot {
    pub entities: Vec<Rc<Entity>>,
    // All invariants must be valid, so no need to save.
}

impl Floor {
    pub fn new() -> Self {
        Floor {
            entities: Vec::new(),
            occupiers: HashMap::new(),
        }
    }

    pub fn get_snapshot(&self) -> FloorSnapshot {
        FloorSnapshot {
            entities: self.entities.clone(),
        }
    }

    // not necessarily the fastest but its safe.
    pub fn from_snapshot(snapshot: FloorSnapshot) -> Self {
        let mut out = Floor::new();
        for entity in snapshot.entities {
            out.add_entity(entity);
        }
        out
    }

    pub fn add_entity(&mut self, new: Rc<Entity>) -> FloorSnapshot {
        let snapshot = self.get_snapshot();

        self.entities.push(Rc::clone(&new));
        match self.occupiers.entry(new.x) {
            std::collections::hash_map::Entry::Occupied(_) => panic!("AHHHHHHH"),
            std::collections::hash_map::Entry::Vacant(vacancy) => {
                vacancy.insert(Rc::clone(&new));
            }
        }

        snapshot
    }

    pub fn get_player(&self) -> Rc<Entity> {
        Rc::clone(self.entities.first().unwrap())
    }

    pub fn get_someone(&self) -> Rc<Entity> {
        Rc::clone(self.entities.last().unwrap())
    }

    pub fn update_entity(&mut self, old: Rc<Entity>, new: Rc<Entity>) -> FloorSnapshot {
        let snapshot = self.get_snapshot();

        self.entities = self
            .entities
            .iter()
            .map(|x| Rc::clone(if Rc::ptr_eq(x, &old) { &new } else { x }))
            .collect::<Vec<Rc<Entity>>>();

        self.occupiers.remove_entry(&old.x);
        match self.occupiers.entry(new.x) {
            std::collections::hash_map::Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
            std::collections::hash_map::Entry::Vacant(vacancy) => {
                vacancy.insert(Rc::clone(&new));
            }
        };

        snapshot
    }

    // more permissive than updating one at a time.
    pub fn update_entities(&mut self, map: HashMap<Rc<Entity>, Rc<Entity>>) -> FloorSnapshot {
        let snapshot = self.get_snapshot();

        self.entities = self
            .entities
            .iter()
            .map(|x| Rc::clone(map.get(x).unwrap_or(x)))
            .collect::<Vec<Rc<Entity>>>();

        for old in map.keys() {
            self.occupiers.remove_entry(&old.x);
        }
        for new in map.values() {
            match self.occupiers.entry(new.x) {
                std::collections::hash_map::Entry::Occupied(_) => panic!("AAAAAAAAAAAAA"),
                std::collections::hash_map::Entry::Vacant(vacancy) => {
                    vacancy.insert(Rc::clone(new));
                }
            };
        }

        snapshot
    }
}

pub trait ActionTrait {
    fn verify_action(&self, floor: &Floor, subject_ref: &Rc<Entity>) -> Option<impl CommandTrait>;
}

pub trait CommandTrait {
    fn do_action(self, floor: &mut Floor) -> FloorSnapshot;
}
