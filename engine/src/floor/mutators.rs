use std::collections::HashMap;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::actions::events::FloorEvent;
use crate::actions::events::GetDownedEvent;
use crate::actions::events::MissionFailedEvent;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntitySet;
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
        old_set: &EntitySet,
        new_set: &mut HashMap<EntityId, Entity>,
    ) -> Writer<Self, FloorEvent> {
        let mut out = Writer::new(self.clone());

        for (id, e) in new_set.iter_mut() {
            if e.health <= 0 && !matches!(e.state, EntityState::Downed { .. }) {
                e.state = EntityState::Downed {
                    round_downed: current_round,
                };
                out = out.log(FloorEvent::GetDowned(GetDownedEvent { subject: *id }));
            }
        }

        let downed_party = new_set
            .iter()
            .filter_map(|(id, e)| {
                if e.is_player_friendly && matches!(e.state, EntityState::Downed { .. }) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if !downed_party.is_empty() {
            let alive_party = old_set
                .iter()
                .filter_map(|(id, e)| {
                    if e.is_player_friendly && !downed_party.contains(&id) {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            // Do a better split.
            if let Some((head, tail)) = alive_party.split_first() {
                out = out.log(FloorEvent::MissionFailed(MissionFailedEvent {
                    subject: *head,
                    downed_party,
                }));
                for e in tail {
                    out = out.log(FloorEvent::MissionFailed(MissionFailedEvent {
                        subject: *e,
                        downed_party: Vec::new(),
                    }));
                }
            }
        }

        out
    }
}
