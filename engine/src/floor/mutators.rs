use std::collections::HashMap;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::actions::events::DieEvent;
use crate::actions::events::FloorEvent;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::writer::Writer;

// TODO: Rethink this pattern?
// Mutators edit entity data and log events
// Decorators read and save data and log events
// Both can be stored in the Floor if they need to save their own data.
// All of the may panic currently. Yippee.
// (Maybe accelerants purely manage data and do not log. Eg a sorter for fast range queries. slice on x, then filter on y.)
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct DownedStateMutator;

impl DownedStateMutator {
    pub fn mutate_entities(
        &self,
        current_round: u32,
        new_set: &mut HashMap<EntityId, Entity>,
    ) -> Writer<Self, FloorEvent> {
        let mut out = Writer::new(self.clone());
        for (id, e) in new_set.iter_mut() {
            if e.health <= 0 && !matches!(e.state, EntityState::Downed { .. }) {
                e.state = EntityState::Downed {
                    round_downed: current_round,
                };
                out = out.log(FloorEvent::Die(DieEvent { subject: *id }));
            }
        }
        out
    }
}
