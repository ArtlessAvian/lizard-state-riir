use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use std::ops::Index;
use std::ops::IndexMut;
use std::rc::Rc;

use crate::actions::NullAction;
use crate::actions::SerializeCommandTrait;
use crate::actions::UnaimedAction;
use crate::positional::AbsolutePosition;

/// An opaque index into an EntitySet.
//
// TODO: Remove Default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Archive, Serialize, Deserialize, Default)]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
pub struct EntityId(usize);

impl From<EntityId> for i32 {
    fn from(value: EntityId) -> Self {
        value.0.try_into().unwrap()
    }
}

/// An entity as it exists in a Floor.
/// Not aware of the floor (and therefore of other entities)
///
/// As part of a floor, an Entity has an `id`, a `next_turn` to optionally participate in turntaking,
/// and a `position` and `state.`
///
/// Outside a Floor, entities have a statline (`health`, etc.) and some constant data (`max_health`, etc.).
// TODO: Split into an EntityData. Wrap with Entity when added to a Floor?
// TODO: Make API more like HashMap? Iter should return keys.
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct Entity {
    // TODO: Try removing. Anywhere where this would've been read, (presumably) the caller could have had indirect access, so pass around (Id, Entity) pairs.
    pub id: EntityId,

    // Turntaking and state are highly correlated, and changing one usually implies something about the other.
    // EG: Doing a move, queued or not, should usually unset the queued move.
    // EG: Being dead means you are not participating in turntaking.
    // EG: Taking a hit puts you in hitstun, (unqueues your move if in ok state) AND delays your next action.
    pub state: EntityState,

    pub pos: AbsolutePosition,
    pub health: i8,

    // TODO: AI. Roughly should be a type that tries a sequence of actions, and on success may mutate its own clone and return the FloorUpdate.
    // Should not be wrapped in Option. A "NullAI" should just wait in place forever.

    // Not mutually exclusive with AI.
    // Eg you might switch between controlling entities, or temporarily take control of one.
    pub is_player_controlled: bool,
}

impl Entity {
    pub fn get_actions() -> Vec<UnaimedAction> {
        vec![UnaimedAction::None(Rc::new(NullAction {}))]
    }

    pub fn get_next_turn(&self) -> Option<u32> {
        match self.state {
            EntityState::Ok { next_turn, .. } => Some(next_turn),
            EntityState::Committed { next_turn, .. } => Some(next_turn),
            EntityState::ConfirmCommand { next_turn, .. } => Some(next_turn),
            EntityState::Hitstun { next_turn, .. } => Some(next_turn),
            EntityState::Knockdown { next_turn, .. } => Some(next_turn),
            EntityState::Dead => None,
        }
    }
}

/// An add only collection.
/// Elements are wrapped in Rc for efficient `Clone`ing.
//
// Remove should never be implemented. If something were to be removed,
// mark it as dead or exited and remove it from turntaking.
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct EntitySet(Vec<Rc<Entity>>);

impl EntitySet {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, mut new: Entity) -> EntityId {
        let id = EntityId(self.0.len());
        new.id = id;
        self.0.push(Rc::new(new));
        id
    }

    // TODO: Think about what this is asking.
    pub fn contains(&self, reference: &Rc<Entity>) -> bool {
        self.0
            .binary_search_by_key(&reference.id.0, |e| e.id.0)
            .is_ok()
    }

    pub fn contains_id(&self, id: &EntityId) -> bool {
        self.0.binary_search_by_key(&id.0, |e| e.id.0).is_ok()
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<Entity>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Rc<Entity>> {
        self.0.iter_mut()
    }
}

impl Default for EntitySet {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<EntityId> for EntitySet {
    // Ideally this would be Entity, but then IndexMut would have to return &mut Entity.
    // We could try to get_mut, but we would never get exclusive access.
    // (Remember EntitySet is intended to be cloned heavily.)
    type Output = Rc<Entity>;

    fn index(&self, index: EntityId) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<EntityId> for EntitySet {
    fn index_mut(&mut self, index: EntityId) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

impl<'a> IntoIterator for &'a EntitySet {
    // TODO: Find a way to hide the Rc.
    type Item = &'a Rc<Entity>;
    type IntoIter = std::slice::Iter<'a, Rc<Entity>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut EntitySet {
    // TODO: Find a way to hide the Rc.
    type Item = &'a mut Rc<Entity>;
    type IntoIter = std::slice::IterMut<'a, Rc<Entity>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

/// Logicless container of info.
// TODO: Maybe split into ALIVE and DEAD.
//       ALIVE would contain next_turn, health, and a further breakdown of state.
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum EntityState {
    Ok {
        next_turn: u32,
    },
    /// On the entities next turn, run `queued_command` automatically.
    /// When the entity is hit, it becomes a counterhit.
    Committed {
        next_turn: u32,
        // Rc for Clone.
        queued_command: Rc<dyn SerializeCommandTrait>,
    },
    /// On the entities next turn, the entity may choose to run this command.
    /// Does *NOT* grant counterhit status.
    // TODO: Think if counterhit should be its own enum/bool.
    // TODO: There is a use for confirming one of a set of actions, with counterhit status.
    //       Think about if this is here only to enable "macros." Maybe the caller should
    //       just hold on to the command and repeat it, rather than the command
    //       requeueing itself.
    ConfirmCommand {
        next_turn: u32,
        to_confirm: Rc<dyn SerializeCommandTrait>,
    },
    // Restricted {
    //     next_turn: u32,
    //     // On the entities next turn, action must be chosen from a set.
    //     // restricted_actions: (),
    //     // allow_wait: bool,
    //     // allow_step: bool, // and so on?
    // },

    // Inactionable states below.
    /// On next turn, go into knockdown.
    /// If hit in this state, next_turn gets extended.
    /// If there are no more extensions, go into knockdown immediately.
    Hitstun {
        next_turn: u32,
        extensions: u32, // Counts down to 0.
    },
    Knockdown {
        next_turn: u32,
    },
    Dead,
}
