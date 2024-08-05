pub mod map;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::actions::events::FloorEvent;
use crate::actions::public::WaitAction;
use crate::actions::ActionTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntitySet;
use crate::floor::map::vision::FloorMapVision;
use crate::floor::map::FloorMap;
use crate::positional::AbsolutePosition;
use crate::writer::Writer;

#[derive(Debug, PartialEq, Eq)]
pub enum TurntakingError {
    PlayerTurn { who: EntityId },
    NoTurntakers,
}

// TODO: Move this into the actions module?
// TODO: Also consider aliasing `EntityUpdate` to `Writer<Entity, FloorEvent>` or `Writer<Vec<Entity>, FloorEvent>`
// HOWEVER. Do not do `Writer<(&Floor, Vec<Entity>), FloorEvent>` to allow lazy Floor::update_entities.
// This encourages moving through invalid states and makes it hard to debug on panic.
// Eager validation is a feature.

pub type UnitUpdate = Writer<(), FloorEvent>;
pub type FloorUpdate = Writer<Floor, FloorEvent>;
pub type BorrowedFloorUpdate<'a> = Writer<&'a Floor, FloorEvent>;

// Proposal:
// Systems enforce invariants and process data. (See existing FloorMapVision.)
// Floor does not implement FloorSystem since it takes ownership unlike systems.
// The generic lets systems read from other systems. Cycles of systems will be impossible to write.
// TODO: Iron out the arguments. Also Floor's arguments. Also wrap in Result?
// TODO: Make systems mutable? Use &mut self for arguments and return &mut Self.
// trait FloorSystem<OtherSystems>: Sized {
//     fn add_entity(&self, new: &Rc<Entity>, other: OtherSystems) -> Writer<Self, FloorEvent>;
//     fn set_map(&self, map: FloorMap, other: OtherSystems) -> Writer<Self, FloorEvent>;
//     fn update_entity(&self, new: &Rc<Entity>, other: OtherSystems) -> Writer<Self, FloorEvent>;
//     fn update_entities(
//         &self,
//         new_set: &[Rc<Entity>],
//         other: OtherSystems,
//     ) -> Writer<Self, FloorEvent>;
// }

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct Floor {
    // Rc is shared between Floor generations.
    // Prefer to use indices since serializing Rcs does not preserve identity.
    pub entities: EntitySet,
    pub occupiers: HashMap<AbsolutePosition, EntityId>,
    pub map: FloorMap,

    // TODO: Wrap current behavior with inner class?
    // Outer class/enum can select between this or a full vision mode. (or literally nothing? for testing?)
    pub vision: Option<FloorMapVision>,
}

impl Floor {
    #[must_use]
    pub fn new() -> Self {
        Floor {
            entities: EntitySet::new(),
            occupiers: HashMap::new(),
            map: FloorMap::new(),
            vision: Some(FloorMapVision::new()),
        }
    }

    // TODO: Consider returning Writer<(Floor, EntityId), FloorEvent>
    #[must_use]
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

        let vision_update = self.vision.as_ref().map_or(Writer::new(None), |x| {
            x.add_entity(&next_entities[id], &self.map).map(Some)
        });

        (
            vision_update.bind(|next_vision| {
                FloorUpdate::new(Floor {
                    entities: next_entities,
                    occupiers: next_occupiers,
                    map: self.map.clone(),
                    vision: next_vision,
                })
            }),
            id,
        )
    }

    #[must_use]
    pub fn set_map(&self, map: FloorMap) -> Self {
        for entity in &self.entities {
            assert!(
                map.is_tile_floor(&entity.pos),
                "Updated map has wall over existing entity."
            );
        }

        Floor {
            entities: self.entities.clone(),
            occupiers: self.occupiers.clone(),
            map,
            vision: self.vision.clone(), // TODO: determine if this makes sense?
        }
    }

    #[must_use]
    pub fn update_entity(&self, new: Entity) -> FloorUpdate {
        let old = &self.entities[new.id];

        let mut next_entities = self.entities.clone();
        next_entities.overwrite(new);
        let new_ref = &next_entities[old.id];

        let mut next_occupiers = self.occupiers.clone();
        next_occupiers.remove_entry(&old.pos);
        match next_occupiers.entry(new_ref.pos) {
            Entry::Occupied(_) => {
                panic!("Updated entity occupies same position as existing entity.")
            }
            Entry::Vacant(vacancy) => {
                vacancy.insert(new_ref.id);
            }
        };

        assert!(
            self.map.is_tile_floor(&new_ref.pos),
            "Updated entity occupies wall position."
        );

        self.vision
            .as_ref()
            .map_or(Writer::new(None), |x| {
                x.update_entity(new_ref, &self.map).map(Some)
            })
            .bind(|next_vision| {
                FloorUpdate::new(Floor {
                    entities: next_entities,
                    occupiers: next_occupiers,
                    map: self.map.clone(),
                    vision: next_vision,
                })
            })
    }

    #[must_use]
    pub fn update_entities(&self, new_set: Vec<Entity>) -> FloorUpdate {
        let old_set = new_set
            .iter()
            .map(|x| &self.entities[x.id])
            .collect::<Vec<&Entity>>();

        let mut next_entities = self.entities.clone();
        for new in new_set {
            next_entities.overwrite(new);
        }

        let new_ref_set: Vec<&Entity> = old_set.iter().map(|x| &next_entities[x.id]).collect();

        let mut next_occupiers: HashMap<AbsolutePosition, EntityId> = self.occupiers.clone();
        for old in &old_set {
            let remove = next_occupiers.remove(&old.pos);
            assert!(remove.is_some());
        }
        for new in &new_ref_set {
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

        self.vision
            .as_ref()
            .map_or(Writer::new(None), |x| {
                x.update_entities(&new_ref_set, &self.map).map(Some)
            })
            .bind(|next_vision| {
                FloorUpdate::new(Floor {
                    entities: next_entities,
                    occupiers: next_occupiers,
                    map: self.map.clone(),
                    vision: next_vision,
                })
            })
    }

    // TODO: Return set of entities?
    // Alternatively, add "pass turn to partner."
    // I don't think NPCs should *need* to reorder their turns, its cool if its in, its whatever if it isn't.
    #[must_use]
    pub fn get_next_entity(&self) -> Option<EntityId> {
        return self
            .entities
            .iter()
            .filter(|e| e.get_next_turn().is_some())
            .min_by_key(|e| e.get_next_turn())
            .map(|e| e.id);
    }

    // If there are no turntaking entities, the next turn can safely be 0 without "going back in time".
    #[must_use]
    pub fn get_current_turn(&self) -> u32 {
        self.get_next_entity()
            .and_then(|e| self.entities[e].get_next_turn())
            .unwrap_or(0)
    }

    /// # Errors
    ///
    /// Will return `TurntakingError::NoTurntakers` if no entity can take a turn.
    /// Will return `TurntakingError::PlayerTurn` if player turn.
    // TODO: Make an error enum. Figure out where to scope it lol.
    // Also this just generally feels bad/inconsistent as an API?
    // To make player turns (with no checking if its your turn???), you [do whatever and] get a CommandTrait, and execute it on the floor.
    // Maybe we can make it impossible to call `do_action` unless you're the floor.
    // Also maybe we wrap EntityId with something to signify its their turn.
    // This again raises the question, does it need to be your turn to run a Command?
    pub fn take_npc_turn(&self) -> Result<FloorUpdate, TurntakingError> {
        let next_id = self.get_next_entity();
        if next_id.is_none() {
            return Result::Err(TurntakingError::NoTurntakers);
        }
        let next_id = next_id.unwrap();

        // Return early depending on state.
        #[allow(clippy::single_match)]
        match &self.entities[next_id].state {
            crate::entity::EntityState::Committed { queued_command, .. } => {
                return Ok(queued_command.do_action(self));
            }
            _ => {}
        }

        if self.entities[next_id].is_player_controlled {
            return Result::Err(TurntakingError::PlayerTurn { who: next_id });
        }

        // TODO: do something interesting
        let next_floor = WaitAction
            .verify_action(self, next_id)
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

    use crate::entity::EntityState;

    let floor = Floor::new();
    let floor = floor
        .add_entity(Entity {
            id: EntityId::default(),
            state: EntityState::Hitstun {
                next_turn: 100,
                extensions: 3,
            },
            pos: AbsolutePosition::new(101, 101),
            health: 103,
            max_energy: 104,
            energy: 105,
            is_player_controlled: false,
        })
        .0
        .into_both()
        .0;

    let mut serializer = AllocSerializer::<256>::default();
    serializer.serialize_value(&floor).unwrap();

    let bytes = serializer.into_serializer().into_inner();
    let archived = unsafe { rkyv::archived_root::<Floor>(&bytes[..]) };

    let deserialized: Floor = archived
        .deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new())
        .unwrap();

    // Debug doesn't sort the HashMaps so this hack equality check fails.
    // PartialEq can't(?) be derived for Floor due to dyn trait (in EntityState).
    // assert_eq!(format!("{:?}", deserialized), format!("{:?}", floor));

    // Instead we have this "even better" hack. Its permissive of ordering issues.
    // Doesn't tell you where the difference is though.
    let count = |string: &String| {
        let mut out = HashMap::new();
        string.chars().for_each(|x| {
            out.insert(x, out.get(&x).unwrap_or(&0) + 1);
        });
        out
    };

    let (debug_deserialized, debug_original) = (format!("{deserialized:?}"), format!("{floor:?}"));
    let (count_deserialized, count_original) = (count(&debug_deserialized), count(&debug_original));

    assert!(
        count_deserialized == count_original,
        "Diffs in Debug dumps: {:?}\n  left: {:?}\n right: {:?}",
        count_deserialized
            .iter()
            .filter(|(k, v)| count_original[k] != **v)
            .map(|(k, _v)| k)
            .collect::<Vec<&char>>(),
        debug_deserialized,
        debug_original
    );
}
