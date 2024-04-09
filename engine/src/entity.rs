use serde::Deserialize;
use serde::Serialize;

use std::ops::Index;
use std::ops::IndexMut;
use std::rc::Rc;

use crate::actions::ActionTrait;
use crate::actions::NullAction;
use crate::positional::AbsolutePosition;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
// TODO: Remove Default.
pub struct EntityId(usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub next_turn: Option<u8>,

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
#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub fn contains(&self, reference: &Rc<Entity>) -> bool {
        self.0.contains(reference)
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
