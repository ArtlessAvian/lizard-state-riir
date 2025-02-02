use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::actions::events::FloorEvent;
use crate::actions::events::GetDownedEvent;
use crate::actions::events::MissionFailedEvent;
use crate::entity::BatchEntityUpdate;
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
        batch: &mut BatchEntityUpdate,
    ) -> Writer<Self, FloorEvent> {
        let mut out = Writer::new(self.clone());

        let events = batch.map_or_noop_with_side(|id, e| {
            if e.health > 0 {
                return (None, None);
            }
            if matches!(e.state, EntityState::Downed { .. }) {
                return (None, None);
            }
            let mut e = e.clone();
            e.state = EntityState::Downed {
                round_downed: current_round,
            };
            (
                Some(e),
                Some(FloorEvent::GetDowned(GetDownedEvent { subject: *id })),
            )
        });

        out = out.log_each(events);

        let downed_party = batch
            .contextless
            .iter_updated()
            .filter_map(|(id, e)| {
                if e.is_player_friendly && matches!(e.state, EntityState::Downed { .. }) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !downed_party.is_empty() {
            let alive_party = batch
                .iter_latest()
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
