use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;
use std::rc::Rc;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::actions::events::FloorEvent;
use crate::actions::known_serializable::KnownCommand;
use crate::actions::known_serializable::KnownUnaimedAction;
use crate::actions::UnaimedAction;
use crate::positional::AbsolutePosition;
use crate::strategy::Strategy;
use crate::writer::Writer;

/// An opaque index into an `EntitySet`.
//
// TODO: Remove Default.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
    Default,
)]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
pub struct EntityId(usize);

/// An entity as it exists in a Floor.
/// Not aware of the floor (and therefore of other entities)
///
/// Outside a Floor, entities have a statline (`health`, etc.) and some constant data (`max_health`, etc.).
// TODO: Split into an EntityData. Wrap with Entity when added to a Floor?
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[cfg_attr(test, derive(Default))]
pub struct Entity {
    // Turntaking and state are highly correlated, and changing one usually implies something about the other.
    // EG: Doing a move, queued or not, should usually unset the queued move.
    // EG: Being downed means you are not participating in turntaking.
    // EG: Taking a hit puts you in hitstun, (unqueues your move if in ok state) AND delays your next action.
    pub state: EntityState,

    pub pos: AbsolutePosition,
    pub health: i8,

    pub max_energy: i8,
    pub energy: i8,

    pub moveset: Vec<KnownUnaimedAction>,

    // TODO: AI. Roughly should be a type that tries a sequence of actions, and on success may mutate its own clone and return the FloorUpdate.
    // Should not be wrapped in Option. A "NullAI" should just wait in place forever.
    pub strategy: Strategy,

    // Not mutually exclusive with AI.
    // Eg you might switch between controlling entities, or temporarily take control of one.
    pub is_player_controlled: bool,

    // Controls aggression. Also controls vision.
    pub is_player_friendly: bool,

    // Some string we don't care about in logic.
    pub payload: String,
}

impl Entity {
    #[must_use]
    // TODO: Change return type to slice or ref.
    pub fn get_actions(&self) -> Vec<UnaimedAction> {
        // We might add more states, so consider using match instead of if let.
        if let EntityState::RestrictedActions {
            restricted_actions, ..
        } = &self.state
        {
            restricted_actions
                .iter()
                .map(|x| UnaimedAction::from(x.clone()))
                .collect()
        } else {
            self.moveset
                .clone()
                .into_iter()
                .map(UnaimedAction::from)
                .collect()
        }
    }

    #[must_use]
    pub fn get_command_to_confirm(&self) -> Option<KnownCommand> {
        if let EntityState::ConfirmCommand { to_confirm, .. } = &self.state {
            Some(to_confirm.clone())
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn get_next_round(&self) -> Option<u32> {
        match self.state {
            EntityState::Ok { next_round, .. }
            | EntityState::Committed { next_round, .. }
            | EntityState::ConfirmCommand { next_round, .. }
            | EntityState::RestrictedActions { next_round, .. }
            | EntityState::Hitstun { next_round, .. }
            | EntityState::Knockdown { next_round, .. } => Some(next_round),
            EntityState::Downed { .. } | EntityState::Exited { .. } => None,
        }
    }

    #[must_use]
    pub(crate) fn get_occupied_position(&self) -> Option<AbsolutePosition> {
        match self.state {
            EntityState::Ok { .. }
            | EntityState::Committed { .. }
            | EntityState::ConfirmCommand { .. }
            | EntityState::RestrictedActions { .. }
            | EntityState::Hitstun { .. } => Some(self.pos),
            EntityState::Knockdown { .. }
            | EntityState::Downed { .. }
            | EntityState::Exited { .. } => None,
        }
    }

    #[must_use]
    pub(crate) fn is_allied(&self, other: &Entity) -> bool {
        self.is_player_friendly == other.is_player_friendly
    }

    #[must_use]
    pub fn get_payload(&self) -> &str {
        &self.payload
    }
}

/// An add only collection.
/// Elements are wrapped in Rc for efficient `Clone`ing.
//
// Remove should never be implemented. If something were to be removed,
// mark it as unalive or exited and remove it from turntaking.
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct EntitySet(Vec<Rc<Entity>>);

impl EntitySet {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn add(&mut self, new: Entity) -> EntityId {
        let id = EntityId(self.0.len());
        self.0.push(Rc::new(new));
        id
    }

    pub(crate) fn overwrite(&mut self, id: EntityId, value: Entity) {
        // Avoid creating a new Rc and dropping the old one.
        if let Some(slot) = Rc::get_mut(&mut self.0[id.0]) {
            *slot = value;
        } else {
            self.0[id.0] = Rc::new(value);
        }
    }

    #[must_use]
    pub(crate) fn contains_id(&self, id: EntityId) -> bool {
        (0..self.0.len()).contains(&id.0)
    }

    #[must_use]
    // Leaky abstraction.
    // TODO: Determine if this actually is needed, if you for some reason want to hold
    // a reference to an entity ignoring lifetimes/scoping.
    pub fn index_as_rc(&self, index: EntityId) -> &Rc<Entity> {
        &self.0[index.0]
    }

    pub(crate) fn iter_entities(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter().map(Rc::as_ref)
    }

    pub fn iter_ids(&self) -> impl Iterator<Item = EntityId> {
        (0..self.0.len()).map(EntityId)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (EntityId, &Entity)> {
        self.0
            .iter()
            .enumerate()
            .map(|(index, element)| (EntityId(index), element.as_ref()))
    }

    #[expect(unused)]
    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<Rc<Entity>> {
        self.0.iter_mut()
    }
}

impl Default for EntitySet {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<EntityId> for EntitySet {
    // We cannot implement IndexMut, since that must return a &mut Entity, which is contained
    // in the Rc. This makes sense, since they are intended to be immutable. Instead, clone
    // and mutate the clone, then overwrite.
    type Output = Entity;

    fn index(&self, index: EntityId) -> &Self::Output {
        &self.0[index.0]
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
// TODO: Maybe split into ALIVE and UNALIVE.
//       ALIVE would contain next_turn, health, and a further breakdown of state.
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum EntityState {
    Ok {
        next_round: u32,
    },
    /// On the entities next turn, run `queued_command` automatically.
    /// When the entity is hit, it becomes a counterhit.
    Committed {
        next_round: u32,
        // Rc for Clone.
        queued_command: KnownCommand,
    },
    /// On the entities next turn, the entity may choose to run this command.
    /// Does *NOT* grant counterhit status.
    // TODO: Think if counterhit should be its own enum/bool.
    // TODO: There is a use for confirming one of a set of actions, with counterhit status.
    //       Think about if this is here only to enable "macros." Maybe the caller should
    //       just hold on to the command and repeat it, rather than the command
    //       requeueing itself.
    ConfirmCommand {
        next_round: u32,
        to_confirm: KnownCommand,
    },
    RestrictedActions {
        next_round: u32,
        // On the entities next turn, action must be chosen from a set.
        restricted_actions: Vec<KnownUnaimedAction>,
    },

    // Inactionable states below. Forces an automatic action handled automatically.
    /// On next turn, go into knockdown.
    /// If hit in this state, `next_round` gets extended.
    /// If there are no more extensions, go into knockdown immediately.
    Hitstun {
        next_round: u32,
        extensions: u32, // Counts down to 0.
    },
    Knockdown {
        next_round: u32,
    },

    // Terminal states below.
    /// Incapacitated, but not dead.
    /// If any party member is downed, the mission is lost.
    Downed {
        /// Not useful for game logic.
        round_downed: u32,
    },
    /// Exited the Floor. All party memebers need to all exit for the mission to continue.
    Exited {
        round_exited: u32,
    },
}

#[cfg(test)]
impl Default for EntityState {
    fn default() -> Self {
        Self::Ok { next_round: 0 }
    }
}

#[derive(Debug)]
pub struct AlreadyPresent;

/// Write only hashmap. Cannot replace keys.
#[derive(Default)]
pub struct BatchEntityUpdateContextless(pub HashMap<EntityId, Entity>);

impl BatchEntityUpdateContextless {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    #[must_use]
    pub(crate) fn wrap(x: HashMap<EntityId, Entity>) -> Self {
        Self(x)
    }

    /// Stores an `Entity` with an existing `EntityId` to be updated.
    ///
    /// Since it is possible to access raw entities from an `EntitySet`,
    /// care is taken to prevent accessing the same `(EntityId, Entity)` twice, editing them
    /// differently, and adding them to the batch. So, it returns Result.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `EntityId` is already present.
    #[expect(unused)]
    pub(crate) fn insert(&mut self, k: EntityId, v: Entity) -> Result<(), AlreadyPresent> {
        match self.0.entry(k) {
            Entry::Occupied(_) => Err(AlreadyPresent),
            Entry::Vacant(vacant) => {
                vacant.insert(v);
                Ok(())
            }
        }
    }

    #[must_use]
    pub(crate) fn add_context(self, context: &EntitySet) -> BatchEntityUpdate {
        BatchEntityUpdate {
            contextless: self,
            context,
        }
    }

    pub(crate) fn iter_ids(&'_ self) -> impl Iterator<Item = EntityId> + '_ {
        self.0.keys().copied()
    }

    pub(crate) fn iter_updated(&self) -> impl Iterator<Item = (EntityId, &Entity)> {
        self.0.iter().map(|(a, b)| (*a, b))
    }
}

pub struct BatchEntityUpdate<'a> {
    pub contextless: BatchEntityUpdateContextless,
    pub context: &'a EntitySet,
}

impl<'a> BatchEntityUpdate<'a> {
    #[must_use]
    pub(crate) fn new(context: &'a EntitySet) -> Self {
        Self {
            contextless: BatchEntityUpdateContextless::new(),
            context,
        }
    }

    #[must_use]
    pub(crate) fn commit(self) -> EntitySet {
        let mut out = self.context.clone();
        for (new_id, new) in self.contextless.0 {
            out.overwrite(new_id, new);
        }
        out
    }

    #[must_use]
    pub(crate) fn get_latest(&self, k: EntityId) -> &Entity {
        self.contextless.0.get(&k).unwrap_or(self.context.index(k))
    }

    /// Applies `f` to the `Entity` with id `k` and stores the result.
    pub(crate) fn apply_and_insert<F>(&mut self, k: EntityId, f: F) -> &mut Self
    where
        F: FnOnce(&Entity) -> Entity,
    {
        self.contextless.0.insert(k, f(self.get_latest(k)));
        self
    }

    /// Applies `f` to the `Entity` with id `k` and stores the result.
    pub(crate) fn apply_and_insert_writer<F>(&mut self, k: EntityId, f: F) -> Writer<(), FloorEvent>
    where
        F: FnOnce(&Entity) -> Writer<Entity, FloorEvent>,
    {
        f(self.get_latest(k)).map(|v| {
            self.contextless.0.insert(k, v);
        })
    }

    // There is no `apply_and_insert_or_noop` since you are usually sure you want to
    // mutate the `Entity` with id `k`.

    /// Applies `f` to *every* `Entity` and only stores the results if Some.
    /// Each entity is processed independent from each other.
    /// Order is unspecified.
    #[expect(unused)]
    pub(crate) fn map_or_noop<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&EntityId, &Entity) -> Option<Entity> + Copy,
    {
        for k in self.context.iter_ids() {
            if let Some(some) = f(&k, self.get_latest(k)) {
                self.contextless.0.insert(k, some);
            }
        }
        self
    }

    /// Applies `f` to *every* `Entity` and stores the results if Some.
    /// Each updated entity is independent from each other.
    /// Order is unspecified.
    pub(crate) fn map_or_noop_writer<F>(&mut self, f: F) -> Writer<(), FloorEvent>
    where
        F: FnOnce(&EntityId, &Entity) -> Writer<Option<Entity>, FloorEvent> + Copy,
    {
        let mut out = Writer::new(());
        for k in self.context.iter_ids() {
            out = out.take_writer(f(&k, self.get_latest(k)).map(|opt| {
                if let Some(some) = opt {
                    self.contextless.0.insert(k, some);
                }
            }));
        }
        out
    }

    pub(crate) fn iter_old(&self) -> impl Iterator<Item = (EntityId, &Entity)> {
        self.contextless
            .iter_ids()
            .map(|id| (id, &self.context[id]))
    }

    #[expect(unused)]
    pub(crate) fn iter_diffs(&self) -> impl Iterator<Item = (EntityId, (&Entity, &Entity))> {
        self.contextless
            .iter_updated()
            .map(|(id, new)| (id, (self.context.index(id), new)))
    }

    pub(crate) fn iter_latest(&self) -> impl Iterator<Item = (EntityId, &Entity)> {
        self.context.iter_ids().map(|k| (k, self.get_latest(k)))
    }
}

#[cfg(test)]
mod test {
    use super::Entity;
    use super::EntitySet;

    #[test]
    fn entity_set_iters() {
        let mut entities = EntitySet::new();
        entities.add(Entity::default());
        entities.add(Entity::default());

        for (pair, other) in entities
            .iter()
            .zip(entities.iter_ids().zip(entities.iter_entities()))
        {
            assert_eq!(pair.0, other.0);
            assert!(std::ptr::eq(pair.1, other.1));
        }
    }
}
