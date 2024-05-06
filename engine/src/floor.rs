use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use crate::actions::events::FloorEvent;
use crate::actions::events::SeeMapEvent;
use crate::actions::public::StepAction;
use crate::actions::ActionTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntitySet;
use crate::positional::fov::StrictFOV;
use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;
use crate::writer::Writer;

// TODO: Decide whether to use non_exhaustive.
#[derive(Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[non_exhaustive]
pub enum FloorTile {
    FLOOR,
    WALL,
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
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
        // HACK: StrictFOV doesn't make sense for vision. You can *infer* extra data (what is/isn't a wall) from what is returned.
        // HACK: Avoid expensive construction on every call!
        let fov: StrictFOV = StrictFOV::new(5);
        let mut tiles = fov.get_field_of_view_tiles(*pos, 5, |x| !self.is_tile_floor(&x));
        // honestly this probably makes this slower for small radius
        tiles.sort_by_key(|x| (x.x, x.y));
        tiles.dedup();

        let mut out: HashMap<AbsolutePosition, FloorTile> = HashMap::new();
        for tile in tiles {
            out.insert(tile, self.get_tile(&tile).clone());
        }
        out
    }
}

impl Default for FloorMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct FloorMapVision {
    // entity_last_at: HashMap<EntityId, AbsolutePosition>,
    // map_vision: HashMap<EntityId, HashMap<AbsolutePosition, FloorTile>>,
    // map_history: HashMap<AbsolutePosition, FloorTile>,
}

impl FloorMapVision {
    fn new() -> Self {
        Self {
            // entity_last_at: HashMap::new(),
            // map_vision: HashMap::new(),
            // map_history: HashMap::new(),
        }
    }

    // TODO: Think about if this is an abuse of Writer?
    // We could either return (FloorMapVision, FloorEvent) tuples or Writer<FloorMapVision, FloorEvent>.
    // There will be multiple "systems" like this struct that do the same.

    // For the tuple, we can create a new Writer<()>, log the event (and other events),
    // then bind () -> FloorUpdate (in a lambda capturing the updated Vision and other struct elements).
    // I have no opinion on this. Its ok. Honestly this is probably better.

    // For the writer, we would obviously want to bind Writer<T> on this function. But what do we do with the T?
    // (T would probably represent a Partial<Floor> like Typescript. Or, a tuple/map with a subset of the struct elements, including empty tuple.)
    // We could decompose Writer<T> to (T, Writer<()>) before binding, but that would just be the first thing we discussed.
    // TBH this does let us return multiple events for free. I will probably do this anyways.

    // We'd need a something like `writer.bind(|t| vision.add_entity(t, ...))` but then everything here must be generic over T and do nothing with it.
    // We could add a method to writer to accumulate a big tuple involving T and the output, like
    // `Writer::accumulate<Passthru, U, F: FnOnce(T) -> (Passthru, U)>(f: F) -> Writer<(Passthru, U), Payload>`
    // but that feels ideologically bad. This is already possible with the current functions but ugly.
    // `writer.bind(|t| {let u, log = f().into_pair(); let mut out = Writer::new((t, u)); for el in log {out = out.log(el)}; out})`
    // Pain.
    // (Speaking of ideology, maybe Partial<Floor> is a monoid. Python's dict update is associative. Empty partial is then an identity.)
    // (Then we can make a different Writer type that uses Partial instead of Vec as the monoid.)
    // We can explicitly make a typestate builder for T. Again there'd be a lot of info passed through the function with nothing to do with it.
    // (With this and the above parenthesized idea, maybe typestate T is just a unit with no data so it doesn't get passed through.)
    // (The functions still take T and return U, so we can be sure after binding everything, the partial contains everything needed.)
    // (We'd still need a log of events itd be nested Writer shenanigans.)

    // I am a smug nerd.

    fn add_entity(&self, new: &Rc<Entity>, map: &FloorMap) -> UnitUpdate {
        let out = Writer::new(());
        out.log(FloorMapVision::new_see_map_event(new.as_ref(), map))
    }

    fn update_entity(&self, new: &Rc<Entity>, map: &FloorMap) -> UnitUpdate {
        let out = Writer::new(());
        out.log(FloorMapVision::new_see_map_event(new.as_ref(), map))
    }

    fn update_entities(&self, new_set: &Vec<Rc<Entity>>, map: &FloorMap) -> UnitUpdate {
        let mut out = Writer::new(());
        for new in new_set {
            out = out.log(FloorMapVision::new_see_map_event(new.as_ref(), map))
        }
        out
    }

    fn new_see_map_event(subject: &Entity, map: &FloorMap) -> FloorEvent {
        FloorEvent::SeeMap(SeeMapEvent {
            subject: subject.id,
            vision: FloorMapVision::get_vision(map, &subject.pos),
        })
    }

    fn get_vision(map: &FloorMap, pos: &AbsolutePosition) -> HashMap<AbsolutePosition, FloorTile> {
        // HACK: StrictFOV doesn't make sense for vision. You can *infer* extra data (what is/isn't a wall) from what is returned.
        // HACK: Avoid expensive construction on every call!
        let fov: StrictFOV = StrictFOV::new(5);
        let mut tiles = fov.get_field_of_view_tiles(*pos, 5, |x| !map.is_tile_floor(&x));
        // honestly this probably makes this slower for small radius
        tiles.sort_by_key(|x| (x.x, x.y));
        tiles.dedup();

        let mut out: HashMap<AbsolutePosition, FloorTile> = HashMap::new();
        for tile in tiles {
            out.insert(tile, map.get_tile(&tile).clone());
        }
        out
    }
}

// TODO: Move this into the actions module?
// TODO: Also consider aliasing `EntityUpdate` to `Writer<Entity, FloorEvent>` or `Writer<Vec<Entity>, FloorEvent>`
// HOWEVER. Do not do `Writer<(&Floor, Vec<Entity>), FloorEvent>` to allow lazy Floor::update_entities.
// This encourages moving through invalid states and makes it hard to debug on panic.
// Eager validation is a feature.

pub type UnitUpdate = Writer<(), FloorEvent>;
pub type FloorUpdate = Writer<Floor, FloorEvent>;
pub type BorrowedFloorUpdate<'a> = Writer<&'a Floor, FloorEvent>;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct Floor {
    // Rc is shared between Floor generations.
    // Prefer to use indices since serializing Rcs does not preserve identity.
    pub entities: EntitySet,
    pub occupiers: HashMap<AbsolutePosition, EntityId>,
    pub map: FloorMap,
    pub vision: FloorMapVision,
}

impl Floor {
    pub fn new() -> Self {
        Floor {
            entities: EntitySet::new(),
            occupiers: HashMap::new(),
            map: FloorMap::new(),
            vision: FloorMapVision::new(),
        }
    }

    // TODO: Consider returning Writer<(Floor, EntityId), FloorEvent>
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

        let update = self.vision.add_entity(&next_entities[id], &self.map);

        (
            update.bind(|_| {
                FloorUpdate::new(Floor {
                    entities: next_entities,
                    occupiers: next_occupiers,
                    map: self.map.clone(),
                    vision: self.vision.clone(),
                })
            }),
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
            vision: self.vision.clone(), // TODO: determine if this makes sense?
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

        let update = self.vision.update_entity(&new, &self.map);

        update.bind(|_| {
            FloorUpdate::new(Floor {
                entities: next_entities,
                occupiers: next_occupiers,
                map: self.map.clone(),
                vision: self.vision.clone(),
            })
        })
    }

    pub fn update_entities(&self, new_set: Vec<Rc<Entity>>) -> FloorUpdate {
        let old_set = new_set
            .iter()
            .map(|x| &self.entities[x.id])
            .collect::<Vec<&Rc<Entity>>>();

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

        let update = self.vision.update_entities(&new_set, &self.map);

        update.bind(|_| {
            FloorUpdate::new(Floor {
                entities: next_entities,
                occupiers: next_occupiers,
                map: self.map.clone(),
                vision: self.vision.clone(),
            })
        })
    }

    // TODO: Return set of entities?
    // Alternatively, add "pass turn to partner."
    // I don't think NPCs should *need* to reorder their turns, its cool if its in, its whatever if it isn't.
    pub fn get_next_entity(&self) -> Option<EntityId> {
        return self
            .entities
            .iter()
            .filter(|e| e.next_turn.is_some())
            .min_by_key(|e| e.next_turn)
            .map(|e| e.id);
    }

    // TODO: Make an error enum. Figure out where to scope it lol.
    // Also this just generally feels bad/inconsistent as an API?
    // To make player turns (with no checking if its your turn???), you [do whatever and] get a CommandTrait, and execute it on the floor.
    // Maybe we can make it impossible to call `do_action` unless you're the floor.
    // Also maybe we wrap EntityId with something to signify its their turn.
    // This again raises the question, does it need to be your turn to run a Command?
    #[allow(clippy::result_unit_err)]
    pub fn take_npc_turn(&self) -> Result<FloorUpdate, ()> {
        let next_id = self.get_next_entity();
        if next_id.is_none() {
            return Result::Err(());
        }
        let next_id = next_id.unwrap();

        // Return early depending on state.
        match &self.entities[next_id].state {
            crate::entity::EntityState::Ok {
                queued_command: Some(queued),
            } => {
                return Ok(queued.do_action(self));
            }
            crate::entity::EntityState::Ok {
                queued_command: None,
            } => {}
            _ => {}
        }

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

#[cfg(test)]
#[test]
// TODO: Decide scope of test.
// (To compile error if floor fails to serialize, deserialize?)
fn serialize_deserialize() {
    use rkyv::ser::{serializers::AllocSerializer, Serializer};

    let floor = Floor::new();
    let floor = floor
        .add_entity(Entity {
            id: EntityId::default(),
            next_turn: Some(100),
            state: crate::entity::EntityState::Hitstun,
            pos: AbsolutePosition::new(101, 101),
            health: 103,
        })
        .0
        .into_both()
        .0;

    let mut serializer = AllocSerializer::<256>::default();
    serializer.serialize_value(&floor).unwrap();

    let bytes = dbg!(serializer.into_serializer().into_inner());
    let archived = unsafe { rkyv::archived_root::<Floor>(&bytes[..]) };

    let deserialized: Floor = dbg!(archived
        .deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new())
        .unwrap());

    assert_eq!(format!("{:?}", deserialized), format!("{:?}", floor));
}
