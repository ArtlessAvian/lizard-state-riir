use std::collections::hash_map::Entry;
use std::collections::HashMap;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::entity::Entity;
use crate::entity::EntityId;
use crate::positional::AbsolutePosition;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct Occupiers(HashMap<AbsolutePosition, EntityId>);

impl Occupiers {
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[must_use]
    pub fn add_entity(&self, new: (EntityId, &Entity)) -> Self {
        let mut clone = self.clone();

        if let Some(occupied) = new.1.get_occupied_position() {
            match clone.0.entry(occupied) {
                Entry::Occupied(_) => {
                    panic!("New entity occupies same position as existing entity.")
                }
                Entry::Vacant(vacancy) => {
                    vacancy.insert(new.0);
                }
            }
        }

        clone
    }

    #[must_use]
    pub fn update_entities(
        &self,
        old_set: &Vec<(EntityId, &Entity)>,
        new_set: &Vec<(EntityId, &Entity)>,
    ) -> Self {
        let mut clone = self.clone();

        for (id, old) in old_set {
            if let Some(pos) = old.get_occupied_position() {
                let remove = clone.0.remove(&pos);
                assert!(remove.is_some_and(|x| x == *id));
            }
        }
        for (id, new) in new_set {
            if let Some(pos) = new.get_occupied_position() {
                match clone.0.entry(pos) {
                    Entry::Occupied(_) => {
                        panic!("Updated entities occupy same position as another entity.")
                    }
                    Entry::Vacant(vacancy) => {
                        vacancy.insert(*id);
                    }
                }
            }
        }

        clone
    }

    #[must_use]
    pub fn get(&self, tile: AbsolutePosition) -> Option<EntityId> {
        self.0.get(&tile).copied()
    }
}

impl Default for Occupiers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[test]
#[should_panic(expected = "New entity occupies same position as existing entity.")]
fn add_panic() {
    use crate::entity::EntitySet;

    let mut entities = EntitySet::new();
    let occupiers = Occupiers::new();

    let first = entities.add(Entity {
        state: crate::entity::EntityState::Ok { next_turn: 1 },
        pos: AbsolutePosition::new(10, 10),
        ..Default::default()
    });
    let occupiers = occupiers.add_entity((first, &entities[first]));

    let second = entities.add(Entity {
        state: crate::entity::EntityState::Ok { next_turn: 1 },
        pos: entities[first].pos,
        ..Default::default()
    });
    let _should_panic = occupiers.add_entity((second, &entities[second]));
}

#[cfg(test)]
#[test]
#[should_panic(expected = "Updated entities occupy same position as another entity.")]
fn update_panic() {
    use crate::entity::EntitySet;
    use crate::entity::EntityState;

    let mut entities = EntitySet::new();
    let occupiers = Occupiers::new();

    let first = entities.add(Entity {
        state: EntityState::Ok { next_turn: 1 },
        pos: AbsolutePosition::new(10, 10),
        ..Default::default()
    });
    let occupiers = occupiers.add_entity((first, &entities[first]));

    let second = entities.add(Entity {
        state: EntityState::Ok { next_turn: 1 },
        pos: AbsolutePosition::new(15, 15),
        ..Default::default()
    });
    let occupiers = occupiers.add_entity((second, &entities[second]));

    let mut second_update = entities[second].clone();
    second_update.pos = entities[first].pos;

    let _should_panic = occupiers.update_entities(
        &vec![(second, &entities[second])],
        &vec![(second, &second_update)],
    );
}

#[cfg(test)]
#[test]
fn update_knockdown() {
    use crate::entity::EntitySet;
    use crate::entity::EntityState;

    let mut entities = EntitySet::new();
    let occupiers = Occupiers::new();

    let first = entities.add(Entity {
        state: EntityState::Ok { next_turn: 1 },
        pos: AbsolutePosition::new(10, 10),
        ..Default::default()
    });
    let occupiers = occupiers.add_entity((first, &entities[first]));

    let second = entities.add(Entity {
        state: EntityState::Ok { next_turn: 1 },
        pos: AbsolutePosition::new(15, 15),
        ..Default::default()
    });
    let occupiers = occupiers.add_entity((second, &entities[second]));

    let mut second_update = entities[second].clone();
    second_update.pos = entities[first].pos;
    second_update.state = EntityState::Knockdown { next_turn: 1 };

    assert!(second_update.get_occupied_position().is_none());

    let _occupiers = occupiers.update_entities(
        &vec![(second, &entities[second])],
        &vec![(second, &second_update)],
    );
}
