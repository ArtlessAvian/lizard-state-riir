use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use std::collections::HashMap;
use std::rc::Rc;

use crate::actions::events::FloorEvent;
use crate::actions::events::SeeMapEvent;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::floor::map::FloorMap;
use crate::floor::map::FloorTile;
use crate::positional::fov::StrictFOV;
use crate::positional::AbsolutePosition;
use crate::writer::Writer;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct FloorMapVision {
    entity_last_at: HashMap<EntityId, AbsolutePosition>,
    map_vision: HashMap<EntityId, HashMap<AbsolutePosition, FloorTile>>,
    map_history: HashMap<AbsolutePosition, FloorTile>,
}

impl FloorMapVision {
    pub fn new() -> Self {
        Self {
            entity_last_at: HashMap::new(),
            map_vision: HashMap::new(),
            map_history: HashMap::new(),
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

    pub fn add_entity(
        &self,
        new: &Rc<Entity>,
        map: &FloorMap,
    ) -> Writer<FloorMapVision, FloorEvent> {
        self.update_entity(new, map)
    }

    pub fn update_entity(
        &self,
        new: &Rc<Entity>,
        map: &FloorMap,
    ) -> Writer<FloorMapVision, FloorEvent> {
        self.update_entities(&vec![new.clone()], map)
    }

    pub fn update_entities(
        &self,
        new_set: &Vec<Rc<Entity>>,
        map: &FloorMap,
    ) -> Writer<FloorMapVision, FloorEvent> {
        let mut out = Writer::new(self.clone());
        for new in new_set {
            if out.get_contents().entity_last_at.get(&new.id) != Some(&new.pos) {
                out = out.bind(|vision| vision.update_and_emit_event(new, map));
            }
        }
        out
    }

    fn update_and_emit_event(
        mut self,
        subject: &Entity,
        map: &FloorMap,
    ) -> Writer<FloorMapVision, FloorEvent> {
        let vision = Self::get_vision(map, &subject.pos);

        self.entity_last_at.insert(subject.id, subject.pos);
        self.map_vision.insert(subject.id, vision.clone());
        for (pos, tile) in &vision {
            self.map_history.insert(*pos, tile.clone());
        }

        Writer::new(self).log(FloorEvent::SeeMap(SeeMapEvent {
            subject: subject.id,
            vision,
        }))
    }

    fn get_vision(map: &FloorMap, pos: &AbsolutePosition) -> HashMap<AbsolutePosition, FloorTile> {
        // HACK: StrictFOV doesn't make sense for vision. You can *infer* extra data (what is/isn't a wall) from what is returned.
        // HACK: Avoid expensive construction on every call!
        let fov: StrictFOV = StrictFOV::new(8);
        let mut tiles = fov.get_field_of_view_tiles(*pos, 8, |x| !map.is_tile_floor(&x));
        // honestly sorting and deduping probably makes this slower for small radius
        tiles.sort();
        tiles.dedup();

        let mut out: HashMap<AbsolutePosition, FloorTile> = HashMap::new();
        for tile in tiles {
            out.insert(tile, map.get_tile(&tile).clone());
        }
        out
    }
}

impl Default for FloorMapVision {
    fn default() -> Self {
        Self::new()
    }
}
