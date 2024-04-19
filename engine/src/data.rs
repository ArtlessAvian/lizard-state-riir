use serde::Deserialize;
use serde::Serialize;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::actions::events::FloorEvent;
use crate::actions::public::StepAction;
use crate::actions::ActionTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntitySet;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

// TODO: Decide whether to use non_exhaustive.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    // TODO: Move responsibility to new struct.
    pub fn get_vision(&self, pos: &AbsolutePosition) -> HashMap<AbsolutePosition, FloorTile> {
        let mut out: HashMap<AbsolutePosition, FloorTile> = HashMap::new();
        for dx in -1..2 {
            for dy in -1..2 {
                let offsetted = *pos + RelativePosition::new(dx, dy);
                out.insert(offsetted, self.get_tile(&offsetted).clone());
            }
        }
        out
    }
}

impl Default for FloorMap {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FloorUpdate(pub Floor, pub Vec<FloorEvent>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Floor {
    // Rc is shared between Floor generations.
    // Prefer to use indices since serializing Rcs does not preserve identity.
    pub entities: EntitySet,
    pub occupiers: HashMap<AbsolutePosition, EntityId>,
    pub map: FloorMap,
}

impl Floor {
    pub fn new() -> Self {
        Floor {
            entities: EntitySet::new(),
            occupiers: HashMap::new(),
            map: FloorMap::new(),
        }
    }

    pub fn add_entity(&self, new: Entity) -> (FloorUpdate, EntityId) {
        let mut next_entities = self.entities.clone();
        let id = next_entities.add(new);

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

        (
            FloorUpdate(
                Floor {
                    entities: next_entities,
                    occupiers: next_occupiers,
                    map: self.map.clone(),
                },
                vec![],
            ),
            id,
        )
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

    pub fn update_entity(&self, new: Rc<Entity>) -> FloorUpdate {
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

        FloorUpdate(
            Floor {
                entities: next_entities,
                occupiers: next_occupiers,
                map: self.map.clone(),
            },
            vec![],
        )
    }

    pub fn update_entities(&self, new_set: HashSet<Rc<Entity>>) -> FloorUpdate {
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

        FloorUpdate(
            Floor {
                entities: next_entities,
                occupiers: next_occupiers,
                map: self.map.clone(),
            },
            vec![],
        )
    }

    pub fn get_next_entity(&self) -> Option<EntityId> {
        return self
            .entities
            .iter()
            .filter(|e| e.next_turn.is_some())
            .min_by_key(|e| e.next_turn)
            .map(|e| e.id);
    }

    pub fn take_npc_turn(&self) -> Result<FloorUpdate, ()> {
        let next_id = self.get_next_entity();
        if next_id.is_none() {
            return Result::Err(());
        }
        let next_id = next_id.unwrap();

        // hardcoded player.
        // TODO: unhardcode. currently hacky behavior with default.
        if next_id == EntityId::default() {
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
