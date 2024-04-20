use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use std::ops::Index;
use std::ops::IndexMut;
use std::rc::Rc;

use crate::actions::ActionTrait;
use crate::actions::CommandTrait;
use crate::actions::NullAction;
use crate::actions::SerializeCommandTrait;
use crate::positional::AbsolutePosition;

/// An opaque index into an EntitySet.
//
// TODO: Remove Default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Archive, Serialize, Deserialize, Default)]
#[archive_attr(derive(Debug))]
pub struct EntityId(usize);

impl From<EntityId> for i32 {
    fn from(value: EntityId) -> Self {
        value.0.try_into().unwrap()
    }
}

/// An entity as it exists in a Floor.
/// Not aware of the floor.
///
/// As part of a floor, an Entity has an `id`, a `next_turn` to optionally participate in turntaking,
/// and a `position` and `state.`
///
/// Outside a Floor, entities have a statline (`health`, etc.) and some constant data (`max_health`, etc.).
// TODO: Split into an EntityData. Wrap with Entity when added to a Floor?
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct Entity {
    pub id: EntityId,
    pub next_turn: Option<u8>,

    pub state: EntityState,
    pub pos: AbsolutePosition,
    pub health: i8,
}

impl Entity {
    pub fn get_actions() -> Box<dyn ActionTrait> {
        Box::new(NullAction {})
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
// TODO: Consider moving turn taking into this?
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub enum EntityState {
    Ok {
        /// On your turn, automatically runs this command.
        // Rc for Clone.
        queued_command: Option<Rc<dyn SerializeCommandTrait>>,
    },
    Hitstun,
    Knockdown,
    Dead,
}
