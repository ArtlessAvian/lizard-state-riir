use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::actions::public::WaitAction;
use crate::actions::ActionTrait;
use crate::entity::EntityId;
use crate::floor::Floor;
use crate::floor::FloorUpdate;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct Strategy {}

impl Strategy {
    #[must_use]
    pub fn take_turn(&self, original: &Floor, subject_id: EntityId) -> FloorUpdate {
        WaitAction {}
            .verify_action(original, subject_id)
            .expect("Wait should never fail")
            .do_action(original)
    }
}
