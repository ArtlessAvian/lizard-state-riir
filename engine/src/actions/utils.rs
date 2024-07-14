use std::rc::Rc;

use crate::entity::EntityId;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::algorithms::Segment;
use crate::positional::RelativePosition;

use super::events::FloorEvent;
use super::events::KnockbackEvent;
use super::CommandTrait;

#[derive(Debug)]
pub struct TakeKnockbackUtil {
    pub entity: EntityId,
    pub vector: RelativePosition,
}

impl CommandTrait for TakeKnockbackUtil {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        // intentionally incorrect, may panic due to invariant breaking lmao
        let mut updated = (*floor.entities[self.entity]).clone();

        let mut last_valid_position = updated.pos;
        for offset in Segment::calculate_relative(self.vector).0 {
            if floor.map.is_tile_floor(&(updated.pos + offset)) {
                last_valid_position = updated.pos + offset;
                // TODO: Knockdown entities in the way.
            } else {
                break;
            }
        }
        updated.pos = last_valid_position;

        floor
            .update_entity(Rc::new(updated))
            .log(FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: self.entity,
                tile: last_valid_position,
            }))
    }
}
