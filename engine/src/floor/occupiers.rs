use std::collections::HashMap;
use std::collections::hash_map::Entry;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::entity::BatchEntityUpdate;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::positional::AbsolutePosition;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
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
    pub fn update_entities(&self, batch: &BatchEntityUpdate) -> Self {
        let mut clone = self.clone();

        for (id, old) in batch.iter_old() {
            if let Some(pos) = old.get_occupied_position() {
                let remove = clone.0.remove(&pos);
                assert!(remove.is_some_and(|x| x == id));
            }
        }
        for (id, new) in batch.contextless.iter_updated() {
            if let Some(pos) = new.get_occupied_position() {
                match clone.0.entry(pos) {
                    Entry::Occupied(_) => {
                        panic!("Updated entities occupy same position as another entity.")
                    }
                    Entry::Vacant(vacancy) => {
                        vacancy.insert(id);
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
mod test {
    use super::Occupiers;
    use crate::entity::BatchEntityUpdate;
    use crate::entity::Entity;
    use crate::entity::EntitySet;
    use crate::positional::AbsolutePosition;

    #[test]
    #[should_panic(expected = "New entity occupies same position as existing entity.")]
    fn add_panic() {
        let mut entities = EntitySet::new();
        let occupiers = Occupiers::new();

        let first = entities.add(Entity {
            state: crate::entity::EntityState::Ok { next_round: 1 },
            pos: AbsolutePosition::new(10, 10),
            ..Default::default()
        });
        let occupiers = occupiers.add_entity((first, &entities[first]));

        let second = entities.add(Entity {
            state: crate::entity::EntityState::Ok { next_round: 1 },
            pos: entities[first].pos,
            ..Default::default()
        });
        let _should_panic = occupiers.add_entity((second, &entities[second]));
    }

    #[test]
    #[should_panic(expected = "Updated entities occupy same position as another entity.")]
    fn update_panic() {
        use crate::entity::EntitySet;
        use crate::entity::EntityState;

        let mut entities = EntitySet::new();
        let occupiers = Occupiers::new();

        let first = entities.add(Entity {
            state: EntityState::Ok { next_round: 1 },
            pos: AbsolutePosition::new(10, 10),
            ..Default::default()
        });
        let occupiers = occupiers.add_entity((first, &entities[first]));

        let second = entities.add(Entity {
            state: EntityState::Ok { next_round: 1 },
            pos: AbsolutePosition::new(15, 15),
            ..Default::default()
        });
        let occupiers = occupiers.add_entity((second, &entities[second]));

        let mut batch = BatchEntityUpdate::new(&entities);
        batch.apply_and_insert(second, |e: &Entity| {
            let mut second_update = e.clone();
            second_update.pos = entities[first].pos;
            second_update
        });
        let _should_panic = occupiers.update_entities(&batch);
    }

    #[test]
    fn update_knockdown() {
        use crate::entity::EntitySet;
        use crate::entity::EntityState;

        let mut entities = EntitySet::new();
        let occupiers = Occupiers::new();

        let first = entities.add(Entity {
            state: EntityState::Ok { next_round: 1 },
            pos: AbsolutePosition::new(10, 10),
            ..Default::default()
        });
        let occupiers = occupiers.add_entity((first, &entities[first]));

        let second = entities.add(Entity {
            state: EntityState::Ok { next_round: 1 },
            pos: AbsolutePosition::new(15, 15),
            ..Default::default()
        });
        let occupiers = occupiers.add_entity((second, &entities[second]));

        let mut batch = BatchEntityUpdate::new(&entities);
        batch.apply_and_insert(second, |e: &Entity| {
            let mut second_update = e.clone();
            second_update.pos = entities[first].pos;
            second_update.state = EntityState::Knockdown { next_round: 1 };
            assert!(second_update.get_occupied_position().is_none());
            second_update
        });

        let _occupiers = occupiers.update_entities(&batch);
    }
}
