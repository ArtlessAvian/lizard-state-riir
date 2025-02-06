use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::actions::events::FloorEvent;
use crate::actions::events::GetDownedEvent;
use crate::actions::events::MissionFailedEvent;
use crate::entity::BatchEntityUpdate;
use crate::entity::Entity;
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
        Writer::new(self.clone())
            .zip_nothing(DownedStateMutator::set_downed_state(current_round, batch))
            .zip_nothing(DownedStateMutator::lose_the_game(batch))
    }

    fn set_downed_state(
        current_round: u32,
        batch: &mut BatchEntityUpdate,
    ) -> Writer<(), FloorEvent> {
        batch.map_or_noop_writer(|id, e| {
            if e.health > 0 {
                return Writer::new(None);
            }
            if matches!(e.state, EntityState::Downed { .. }) {
                return Writer::new(None);
            }
            Writer::new({
                Entity {
                    state: EntityState::Downed {
                        round_downed: current_round,
                    },
                    ..e.clone()
                }
            })
            .map(Some)
            .log(FloorEvent::GetDowned(GetDownedEvent { subject: *id }))
        })
    }

    fn lose_the_game(batch: &BatchEntityUpdate) -> Writer<(), FloorEvent> {
        let downed_party = batch
            .contextless
            .iter_updated()
            .filter_map(|(id, e)| {
                if e.is_player_friendly && matches!(e.state, EntityState::Downed { .. }) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if downed_party.is_empty() {
            return Writer::new(());
        }

        // TODO: Replace with something less silly. Ignoring code quality because of that.

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

        let mut out = Writer::<(), FloorEvent>::new(());

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

        out
    }
}
