use std::rc::Rc;

use super::events::FloorEvent;
use super::events::KnockbackEvent;
use super::CommandTrait;
use super::SerializeCommandTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::algorithms::Segment;
use crate::positional::RelativePosition;

#[derive(Debug)]
pub struct TakeKnockbackUtil {
    pub entity: EntityId,
    pub vector: RelativePosition,
}

impl CommandTrait for TakeKnockbackUtil {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        // intentionally incorrect, may panic due to invariant breaking lmao
        let mut updated = (floor.entities[self.entity]).clone();

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
            .update_entity(updated)
            .log(FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: self.entity,
                tile: last_valid_position,
            }))
    }
}

#[derive(Debug)]
pub struct DelayCommand {
    pub subject_id: EntityId,
    pub queued_command: Rc<dyn SerializeCommandTrait>,
    pub turns: u32,
    pub event: Option<FloorEvent>,
}

impl CommandTrait for DelayCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = floor.entities[self.subject_id].clone();
        subject_clone.state = EntityState::Committed {
            next_turn: floor.get_current_turn() + self.turns,
            queued_command: Rc::clone(&self.queued_command),
        };

        floor
            .update_entity(subject_clone)
            .log_option(self.event.clone())
    }
}
