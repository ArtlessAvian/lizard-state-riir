use super::events::FloorEvent;
use super::events::KnockbackEvent;
use super::events::KnockdownEvent;
use super::static_dispatch::SerializableCommand;
use super::CommandTrait;
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
        // TODO: Cleanup.
        let mut updated = (floor.entities[self.entity]).clone();

        let mut knocked_down = Vec::new();

        let mut last_valid_position = updated.pos;
        for offset in Segment::calculate_relative(self.vector)
            .0
            .into_iter()
            .skip(1)
        {
            if floor.map.is_tile_floor(&(updated.pos + offset)) {
                last_valid_position = updated.pos + offset;
                if let Some(in_the_way) = floor.occupiers.get(updated.pos + offset) {
                    let mut clone = floor.entities[in_the_way].clone();
                    clone.state = EntityState::Knockdown {
                        next_turn: updated
                            .get_next_turn()
                            .expect("Entity taking knockback should be a turntaker."),
                    };
                    knocked_down.push((in_the_way, clone));
                }
            } else {
                break;
            }
        }
        updated.pos = last_valid_position;

        let knocked_down_ids: Vec<_> = knocked_down.iter().map(|x| x.0).collect();

        let mut all_updated = vec![(self.entity, updated)];
        all_updated.append(&mut knocked_down);

        let mut update = floor
            .update_entities(all_updated)
            .log(FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: self.entity,
                tile: last_valid_position,
            }));
        for id in knocked_down_ids {
            update = update.log(FloorEvent::KnockdownEvent(KnockdownEvent { subject: id }));
        }
        update
    }
}

#[derive(Debug)]
pub struct DelayCommand {
    pub subject_id: EntityId,
    pub queued_command: SerializableCommand,
    pub turns: u32,
    pub event: Option<FloorEvent>,
}

impl CommandTrait for DelayCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = floor.entities[self.subject_id].clone();
        subject_clone.state = EntityState::Committed {
            next_turn: floor.get_current_turn() + self.turns,
            queued_command: self.queued_command.clone(),
        };

        floor
            .update_entity((self.subject_id, subject_clone))
            .log_option(self.event.clone())
    }
}
