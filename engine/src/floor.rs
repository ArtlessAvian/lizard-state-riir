pub mod map;
pub mod mutators;
pub mod occupiers;

use std::collections::HashMap;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::actions::events::FloorEvent;
use crate::actions::public::KnockdownAfterJuggleAction;
use crate::actions::public::TryToStandUpAction;
use crate::actions::ActionTrait;
use crate::actions::CommandTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntitySet;
use crate::entity::EntityState;
use crate::floor::map::vision::FloorMapVision;
use crate::floor::map::FloorMap;
use crate::floor::mutators::DownedStateMutator;
use crate::floor::occupiers::Occupiers;
use crate::strategy::StrategyTrait;
use crate::writer::Writer;

#[derive(Debug, PartialEq, Eq)]
pub enum TurntakingError {
    PlayerTurn { who: EntityId },
    NoTurntakers,
}

pub enum FloorEndState {
    Undetermined,
    AnyPartyMemberDowned,
    AllPartyMembersExited,
    ExitedButNoFood,
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
    pub occupiers: Occupiers,
    pub map: FloorMap,

    pub downing: Option<DownedStateMutator>,

    // TODO: Wrap current behavior with inner class?
    // Outer class/enum can select between this or a full vision mode. (or literally nothing? for testing?)
    pub vision: Option<FloorMapVision>,
}

impl Floor {
    #[must_use]
    pub fn new_with_all_systems() -> Self {
        Floor {
            entities: EntitySet::new(),
            occupiers: Occupiers::new(),
            map: FloorMap::new_empty(),
            downing: Some(DownedStateMutator),
            vision: Some(FloorMapVision::new()),
        }
    }

    // For testing. If you want to enable specific things, mutate or construct.
    #[must_use]
    pub fn new_minimal() -> Self {
        Floor {
            entities: EntitySet::new(),
            occupiers: Occupiers::new(),
            map: FloorMap::new_empty(),
            downing: None,
            vision: None,
        }
    }

    // TODO: Consider returning Writer<(Floor, EntityId), FloorEvent>
    #[must_use]
    pub fn add_entity(&self, new: Entity) -> (FloorUpdate, EntityId) {
        let mut next_entities = self.entities.clone();
        let id = next_entities.add(new);

        let next_occupiers = self.occupiers.add_entity((id, &next_entities[id]));

        assert!(
            self.map.is_tile_floor(&next_entities[id].pos),
            "New entity occupies wall position."
        );

        let vision_update = self.vision.as_ref().map_or(Writer::new(None), |x| {
            x.add_entity((id, &next_entities[id]), &self.map).map(Some)
        });

        (
            vision_update.bind(|next_vision| {
                FloorUpdate::new(Floor {
                    entities: next_entities,
                    occupiers: next_occupiers,
                    map: self.map.clone(),
                    downing: self.downing.clone(),
                    vision: next_vision,
                })
            }),
            id,
        )
    }

    #[must_use]
    pub fn set_map(&self, map: FloorMap) -> Self {
        for entity in &self.entities {
            if let Some(pos) = entity.get_occupied_position() {
                assert!(
                    map.is_tile_floor(&pos),
                    "Updated map has wall over existing entity."
                );
            }
        }

        Floor {
            entities: self.entities.clone(),
            occupiers: self.occupiers.clone(),
            map,
            downing: self.downing.clone(),
            vision: self.vision.clone(), // TODO: determine if this makes sense?
        }
    }

    #[must_use]
    pub fn update_entity(&self, new: (EntityId, Entity)) -> FloorUpdate {
        self.update_entities_map(std::iter::once(new).collect())
    }

    // TODO: Figure out nicer API that isn't error prone for the caller.
    // The caller might try update the same entityid twice. This is usually not intended.
    // The preferred action might be to panic. The caller should make sure to update their existing copy if they intend to make multiple updates.
    // We can also create a nice type/api to allow mutability of something like [Floor].entities. Cow maybe?
    #[must_use]
    pub fn update_entities(&self, new_set: Vec<(EntityId, Entity)>) -> FloorUpdate {
        let len = new_set.len();
        let map = new_set.into_iter().collect::<HashMap<EntityId, Entity>>();
        assert!(map.len() == len);
        self.update_entities_map(map)
    }

    // TODO: Figure out nicer API that isn't so annoying for the caller.
    #[must_use]
    pub fn update_entities_map(&self, mut new_set: HashMap<EntityId, Entity>) -> FloorUpdate {
        let old_set = new_set
            .keys()
            .map(|id| (*id, &self.entities[*id]))
            .collect::<Vec<(EntityId, &Entity)>>();

        let log = self.downing.as_ref().map(|some| {
            some.mutate_entities(self.get_current_round(), &self.entities, &mut new_set)
        });

        let mut next_entities = self.entities.clone();
        for (new_id, new) in new_set {
            next_entities.overwrite(new_id, new);
        }

        let new_ref_set: Vec<(EntityId, &Entity)> = old_set
            .iter()
            .map(|(id, _)| (*id, &next_entities[*id]))
            .collect();

        let next_occupiers = self.occupiers.update_entities(&old_set, &new_ref_set);

        for (_, new) in &new_ref_set {
            if let Some(pos) = new.get_occupied_position() {
                assert!(
                    self.map.is_tile_floor(&pos),
                    "Updated entity occupies wall position."
                );
            }
        }

        // TODO: Fix monad manipulation from hell.
        Writer::transpose(log)
            .zip(Writer::transpose(
                self.vision
                    .as_ref()
                    .map(|x| x.update_entities(&new_ref_set, &self.map)),
            ))
            .bind(|(next_dying, next_vision)| {
                FloorUpdate::new(Floor {
                    entities: next_entities,
                    occupiers: next_occupiers,
                    map: self.map.clone(),
                    downing: next_dying,
                    vision: next_vision,
                })
            })
    }

    // TODO: Return set of entities?
    // Alternatively, add "pass turn to partner."
    // I don't think NPCs should *need* to reorder their turns, its cool if its in, its whatever if it isn't.
    #[must_use]
    pub fn get_current_entity(&self) -> Option<EntityId> {
        self.get_current_turn().map(|x| x.1)
    }

    // If there are no turntaking entities, the next turn can safely be 0 without "going back in time".
    #[must_use]
    pub fn get_current_round(&self) -> u32 {
        self.get_current_turn().map_or(0, |x| x.0)
    }

    // TODO: Return set of entities?
    #[must_use]
    pub fn get_current_turn(&self) -> Option<(u32, EntityId)>
    where
        EntityId: Ord,
    {
        self.entities
            .iter()
            .filter_map(|(id, e)| (e.get_next_round().map(|x| (x, id))))
            .min()
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
        let next_id = self
            .get_current_entity()
            .ok_or(TurntakingError::NoTurntakers)?;

        // Return early depending on state.
        match &self.entities[next_id].state {
            EntityState::Committed { queued_command, .. } => {
                return Ok(queued_command.do_action(self));
            }
            EntityState::Knockdown { .. } => {
                return Ok(TryToStandUpAction
                    .verify_action(self, next_id)
                    .expect("only fails if entity is not knockdown state")
                    .do_action(self))
            }
            EntityState::Hitstun { .. } => {
                return Ok(KnockdownAfterJuggleAction
                    .verify_action(self, next_id)
                    .expect("only fails if entity is not hitstun state")
                    .do_action(self))
            }
            EntityState::Downed { .. } | EntityState::Exited { .. } => {
                unreachable!("terminal state entities do not participate in turntaking")
            }
            EntityState::Ok { .. }
            | EntityState::ConfirmCommand { .. }
            | EntityState::RestrictedActions { .. } => (),
        }

        if self.entities[next_id].is_player_controlled {
            return Result::Err(TurntakingError::PlayerTurn { who: next_id });
        }

        // TODO: do something interesting
        Result::Ok(self.entities[next_id].strategy.take_turn(self, next_id))
    }

    #[must_use]
    pub fn get_end_state(&self) -> FloorEndState {
        if self
            .entities
            .iter_entities()
            .any(|e| e.is_player_friendly && matches!(e.state, EntityState::Downed { .. }))
        {
            FloorEndState::AnyPartyMemberDowned
        } else if self
            .entities
            .iter_entities()
            .filter(|e| e.is_player_friendly)
            .all(|e| matches!(e.state, EntityState::Exited { .. }))
        {
            FloorEndState::AllPartyMembersExited
        } else {
            FloorEndState::Undetermined
        }
    }
}

impl Default for Floor {
    fn default() -> Self {
        // Honestly this shouldn't be the default. Maybe the default for library users I guess?
        Floor::new_with_all_systems()
    }
}
