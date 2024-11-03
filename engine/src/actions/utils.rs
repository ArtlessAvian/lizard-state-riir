use std::collections::HashSet;

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
pub struct MultiKnockbackUtil {
    pub all_displacements: Vec<(EntityId, RelativePosition)>,
}

impl CommandTrait for MultiKnockbackUtil {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let now = floor.get_current_turn();

        let each_swept_tiles = self
            .all_displacements
            .iter()
            .map(|(id, displacement)| {
                (
                    *id,
                    Segment::calculate_relative(*displacement)
                        .0
                        .into_iter()
                        .map(|offset| floor.entities[*id].pos + offset)
                        .take_while(|tile| floor.map.is_tile_floor(tile))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        let swept_tiles = each_swept_tiles
            .iter()
            .flat_map(|(_, segment)| segment.iter());

        let knocked_over = swept_tiles
            .filter_map(|tile| floor.occupiers.get(*tile))
            .filter(|id| self.all_displacements.iter().all(|(kb_id, _)| id != kb_id))
            .collect::<HashSet<_>>();

        let after_knockdowns = knocked_over
            .iter()
            .map(|id| {
                let mut clone = floor.entities[*id].clone();
                clone.state = EntityState::Knockdown { next_turn: now };
                (*id, clone)
            })
            .collect::<Vec<_>>();

        let final_positions = each_swept_tiles
            .iter()
            .map(|(id, segment)| {
                (
                    *id,
                    *segment.last().expect(
                        "Entity's current position should be a floor, which is part of the segment",
                    ),
                )
            })
            .collect::<Vec<_>>();

        let conflict_knockdowns = final_positions
            .iter()
            .filter(|(id, pos)| {
                final_positions
                    .iter()
                    .filter(|(other_id, other_pos)| other_id != id && other_pos == pos)
                    .any(|(other_id, _)| {
                        let my_distance = floor.entities[*id].pos.distance(*pos);
                        let other_distance = floor.entities[*other_id].pos.distance(*pos);
                        my_distance
                            .cmp(&other_distance)
                            .then(id.cmp(other_id).reverse())
                            .is_lt()
                        // We can ignore the eq case, since ids should be unique by design (and guaranteed by the previous filter)
                    })
            })
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();

        let after_knockback = final_positions
            .iter()
            .map(|(id, thing)| {
                let mut clone = floor.entities[*id].clone();
                clone.pos = *thing;
                if conflict_knockdowns.contains(id) {
                    clone.state = EntityState::Knockdown { next_turn: now }
                }
                (*id, clone)
            })
            .collect::<Vec<_>>();

        floor
            .update_entities(after_knockdowns)
            .bind(|floor| floor.update_entities(after_knockback))
            .log_each(final_positions.iter().map(|(id, final_position)| {
                FloorEvent::KnockbackEvent(KnockbackEvent {
                    subject: *id,
                    tile: *final_position,
                })
            }))
            .log_each(
                conflict_knockdowns
                    .iter()
                    .map(|id| FloorEvent::KnockdownEvent(KnockdownEvent { subject: *id })),
            )
            .log_each(
                knocked_over
                    .iter()
                    .map(|id| FloorEvent::KnockdownEvent(KnockdownEvent { subject: *id })),
            )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Entity;
    use crate::floor::map::FloorMap;
    use crate::floor::map::FloorTile;
    use crate::floor::Floor;
    use crate::positional::AbsolutePosition;
    use crate::positional::RelativePosition;

    #[test]
    fn knockback_basic() {
        let floor = Floor::default();
        let (update, id) = floor.add_entity(Entity::default());

        let update = update.bind(|floor| {
            TakeKnockbackUtil {
                entity: id,
                vector: RelativePosition::new(1, 0),
            }
            .do_action(&floor)
        });
        assert_eq!(
            update.get_contents().entities[id].pos,
            AbsolutePosition::new(1, 0)
        );
        assert_eq!(
            update.get_log()[0],
            FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: id,
                tile: AbsolutePosition::new(1, 0)
            })
        );

        let update = update.bind(|floor| {
            TakeKnockbackUtil {
                entity: id,
                vector: RelativePosition::new(0, 1),
            }
            .do_action(&floor)
        });
        assert_eq!(
            update.get_contents().entities[id].pos,
            AbsolutePosition::new(1, 1)
        );
        assert_eq!(
            update.get_log()[1],
            FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: id,
                tile: AbsolutePosition::new(1, 1)
            })
        );
    }

    #[test]
    fn knockback_intheway() {
        let floor = Floor::default();
        let (update, id) = floor.add_entity(Entity::default());
        let (update, other) = update.bind_with_side_output(|floor| {
            floor.add_entity(Entity {
                pos: AbsolutePosition::new(1, 0),
                ..Entity::default()
            })
        });

        let update = update.bind(|floor| {
            TakeKnockbackUtil {
                entity: id,
                vector: RelativePosition::new(1, 0),
            }
            .do_action(&floor)
        });
        assert_eq!(
            update.get_contents().entities[id].pos,
            AbsolutePosition::new(1, 0)
        );
        assert_eq!(
            update.get_log()[0],
            FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: id,
                tile: AbsolutePosition::new(1, 0)
            })
        );
        assert!(matches!(
            update.get_contents().entities[other].state,
            EntityState::Knockdown { next_turn: 0 }
        ));
        assert_eq!(
            update.get_log()[1],
            FloorEvent::KnockdownEvent(KnockdownEvent { subject: other })
        );
    }

    #[test]
    fn multiknockback_basic() {
        let floor = Floor::default();
        let (update, id) = floor.add_entity(Entity::default());
        let (update, other) = update.bind_with_side_output(|floor| {
            floor.add_entity(Entity {
                pos: AbsolutePosition::new(1, 0),
                ..Entity::default()
            })
        });

        let update = update.bind(|floor| {
            MultiKnockbackUtil {
                all_displacements: vec![
                    (id, RelativePosition::new(1, 0)),
                    (other, RelativePosition::new(1, 0)),
                ],
            }
            .do_action(&floor)
        });
        // TODO: Remove event ordering strictness.
        assert_eq!(
            update.get_contents().entities[id].pos,
            AbsolutePosition::new(1, 0)
        );
        assert_eq!(
            update.get_contents().entities[other].pos,
            AbsolutePosition::new(2, 0)
        );
        assert_eq!(
            update.get_log()[0],
            FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: id,
                tile: AbsolutePosition::new(1, 0)
            })
        );
        assert_eq!(
            update.get_log()[1],
            FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: other,
                tile: AbsolutePosition::new(2, 0)
            })
        );
    }

    #[test]
    fn multiknockback_intheway() {
        let floor = Floor::default();
        let (update, id) = floor.add_entity(Entity::default());
        let (update, other) = update.bind_with_side_output(|floor| {
            floor.add_entity(Entity {
                pos: AbsolutePosition::new(1, 0),
                ..Entity::default()
            })
        });

        let update = update.bind(|floor| {
            MultiKnockbackUtil {
                all_displacements: vec![(id, RelativePosition::new(1, 0))],
            }
            .do_action(&floor)
        });
        // TODO: Remove event ordering strictness.
        assert_eq!(
            update.get_contents().entities[id].pos,
            AbsolutePosition::new(1, 0)
        );
        assert_eq!(
            update.get_log()[0],
            FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: id,
                tile: AbsolutePosition::new(1, 0)
            })
        );
        assert!(matches!(
            update.get_contents().entities[other].state,
            EntityState::Knockdown { next_turn: 0 }
        ));
        assert_eq!(
            update.get_log()[1],
            FloorEvent::KnockdownEvent(KnockdownEvent { subject: other })
        );
    }

    #[test]
    fn multiknockback_conflicts() {
        let floor = Floor::default();
        let floor = floor.set_map(FloorMap::new_with_tiles(
            [
                (AbsolutePosition::new(0, 0), FloorTile::FLOOR),
                (AbsolutePosition::new(1, 0), FloorTile::FLOOR),
            ]
            .into(),
        ));

        let (update, id) = floor.add_entity(Entity::default());
        let (update, other) = update.bind_with_side_output(|floor| {
            floor.add_entity(Entity {
                pos: AbsolutePosition::new(1, 0),
                ..Entity::default()
            })
        });

        let update = update.bind(|floor| {
            MultiKnockbackUtil {
                all_displacements: vec![
                    (id, RelativePosition::new(1, 0)),
                    (other, RelativePosition::new(1, 0)),
                ],
            }
            .do_action(&floor)
        });
        // TODO: Remove event ordering strictness.
        assert_eq!(
            update.get_contents().entities[id].pos,
            AbsolutePosition::new(1, 0)
        );
        assert_eq!(
            update.get_log()[0],
            FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: id,
                tile: AbsolutePosition::new(1, 0)
            })
        );
        assert_eq!(
            update.get_contents().entities[other].pos,
            AbsolutePosition::new(1, 0)
        );
        assert_eq!(
            update.get_log()[1],
            FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: other,
                tile: AbsolutePosition::new(1, 0)
            })
        );
        assert!(matches!(
            update.get_contents().entities[other].state,
            EntityState::Knockdown { next_turn: 0 }
        ));
        assert_eq!(
            update.get_log()[2],
            FloorEvent::KnockdownEvent(KnockdownEvent { subject: other })
        );
    }
}
