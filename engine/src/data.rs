use serde::Deserialize;
use serde::Serialize;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::actions::public::StepAction;
use crate::actions::ActionTrait;
use crate::actions::NullAction;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub id: usize,
    pub next_turn: Option<u8>,

    pub pos: AbsolutePosition,
    pub health: i8,
}

impl Entity {
    pub fn get_actions() -> Box<dyn ActionTrait> {
        Box::new(NullAction {})
    }
}

// TODO: Decide whether to use non_exhaustive.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FloorTile {
    FLOOR,
    WALL,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloorMap {
    pub tiles: Rc<HashMap<AbsolutePosition, FloorTile>>,
    pub default: FloorTile,
}

impl FloorMap {
    pub fn new() -> Self {
        FloorMap {
            tiles: Rc::new(HashMap::new()),
            default: FloorTile::FLOOR, // "outdoors" map.
        }
    }

    pub fn get_tile(&self, pos: &AbsolutePosition) -> &FloorTile {
        self.tiles.get(pos).unwrap_or(&self.default)
    }

    pub fn is_tile_floor(&self, pos: &AbsolutePosition) -> bool {
        // clean (and obvious) but more floors will be added ig.
        matches!(self.get_tile(pos), FloorTile::FLOOR)
    }
}

impl Default for FloorMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Floor {
    // Rc is shared between Floor generations.
    // Prefer to use indices since serializing Rcs does not preserve identity.
    pub entities: Vec<Rc<Entity>>,
    pub occupiers: HashMap<AbsolutePosition, usize>,
    pub map: FloorMap,
}

impl Floor {
    pub fn new() -> Self {
        Floor {
            entities: Vec::new(),
            occupiers: HashMap::new(),
            map: FloorMap::new(),
        }
    }

    pub fn add_entity(&self, mut new: Entity) -> Self {
        new.id = self.entities.len();
        let id = new.id;

        let mut next_entities = self.entities.clone();
        next_entities.push(Rc::new(new));

        let mut next_occupiers = self.occupiers.clone();
        match next_occupiers.entry(next_entities[id].pos) {
            Entry::Occupied(_) => panic!("New entity occupies same position as existing entity."),
            Entry::Vacant(vacancy) => {
                vacancy.insert(id);
            }
        }

        assert!(
            self.map.is_tile_floor(&next_entities[id].pos),
            "New entity occupies wall position."
        );

        Floor {
            entities: next_entities,
            occupiers: next_occupiers,
            map: self.map.clone(),
        }
    }

    pub fn set_map(&self, map: FloorMap) -> Self {
        for entity in &self.entities {
            assert!(
                map.is_tile_floor(&entity.pos),
                "Updated map has wall over existing entity."
            )
        }

        Floor {
            entities: self.entities.clone(),
            occupiers: self.occupiers.clone(),
            map,
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
            Entry::Occupied(_) => panic!("Updated entity occupy same position as existing entity."),
            Entry::Vacant(vacancy) => {
                vacancy.insert(new.id);
            }
        };

        assert!(
            self.map.is_tile_floor(&new.pos),
            "Updated entity occupies wall position."
        );

        Floor {
            entities: next_entities,
            occupiers: next_occupiers,
            map: self.map.clone(),
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
            let remove = next_occupiers.remove(&old.pos);
            assert!(remove.is_some());
        }
        for new in &new_set {
            match next_occupiers.entry(new.pos) {
                Entry::Occupied(_) => {
                    panic!("Updated entities occupy same position as another entity.")
                }
                Entry::Vacant(vacancy) => {
                    vacancy.insert(new.id);
                }
            }

            assert!(
                self.map.is_tile_floor(&new.pos),
                "Updated entity occupies wall position."
            );
        }

        Floor {
            entities: next_entities,
            occupiers: next_occupiers,
            map: self.map.clone(),
        }
    }

    pub fn get_next_entity(&self) -> Option<usize> {
        return self
            .entities
            .iter()
            .filter(|e| e.next_turn.is_some())
            .min_by_key(|e| e.next_turn)
            .map(|e| e.id);
    }

    pub fn take_npc_turn(&self) -> Result<Floor, ()> {
        let next_id = self.get_next_entity();
        if next_id.is_none() {
            return Result::Err(());
        }
        let next_id = next_id.unwrap();

        // hardcoded player.
        // TODO: unhardcode.
        if next_id == 0 {
            return Result::Err(());
        }

        // TODO: do something interesting
        let next_floor = StepAction {
            dir: RelativePosition { dx: 0, dy: 1 },
        }
        .verify_action(self, &self.entities[next_id])
        .expect("testing code")
        .do_action(self);

        Result::Ok(next_floor)
    }
}

impl Default for Floor {
    fn default() -> Self {
        Floor::new()
    }
}
