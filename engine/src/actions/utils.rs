use std::collections::HashSet;
use std::num::NonZero;

use super::events::FloorEvent;
use super::events::JuggleHitEvent;
use super::events::JuggleLimitEvent;
use super::events::KnockbackEvent;
use super::events::KnockdownEvent;
use super::known_serializable::KnownCommand;
use super::CommandTrait;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::algorithms::Segment;
use crate::positional::RelativePosition;
use crate::writer::Writer;

#[derive(Debug)]
pub struct TakeKnockbackUtil {
    pub entity: EntityId,
    pub vector: RelativePosition,
}

impl CommandTrait for TakeKnockbackUtil {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let now = floor.get_current_round();

        let swept_tiles = Segment::calculate_relative(self.vector)
            .0
            .into_iter()
            .map(|offset| floor.entities[self.entity].pos + offset)
            .take_while(|tile| floor.map.is_tile_floor(tile))
            .collect::<Vec<_>>();

        let knocked_over = swept_tiles
            .iter()
            .filter_map(|tile| floor.occupiers.get(*tile))
            .filter(|id| *id != self.entity)
            .collect::<Vec<_>>();

        let after_knockdowns = knocked_over
            .iter()
            .map(|id| {
                let mut clone = floor.entities[*id].clone();
                clone.state = EntityState::Knockdown { next_round: now };
                (*id, clone)
            })
            .collect::<Vec<_>>();

        let final_position = *swept_tiles
            .last()
            .expect("Entity's current position should be a floor, which is part of the segment");

        let after_knockback = {
            let mut clone = floor.entities[self.entity].clone();
            clone.pos = final_position;
            clone
        };

        floor
            .update_entities(after_knockdowns)
            .bind(|floor| floor.update_entity((self.entity, after_knockback)))
            .log({
                FloorEvent::KnockbackEvent(KnockbackEvent {
                    subject: self.entity,
                    tile: final_position,
                })
            })
            .log_each(
                knocked_over
                    .iter()
                    .map(|id| FloorEvent::KnockdownEvent(KnockdownEvent { subject: *id })),
            )
    }
}

#[derive(Debug)]
pub struct MultiKnockbackUtil {
    pub all_displacements: Vec<(EntityId, RelativePosition)>,
}

impl CommandTrait for MultiKnockbackUtil {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let now = floor.get_current_round();

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
                clone.state = EntityState::Knockdown { next_round: now };
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
                    clone.state = EntityState::Knockdown { next_round: now }
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
    pub queued_command: KnownCommand,
    pub turns: u32,
    pub event: Option<FloorEvent>,
}

impl CommandTrait for DelayCommand {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        let mut subject_clone: Entity = floor.entities[self.subject_id].clone();
        subject_clone.state = EntityState::Committed {
            next_round: floor.get_current_round() + self.turns,
            queued_command: self.queued_command.clone(),
        };

        floor
            .update_entity((self.subject_id, subject_clone))
            .log_option(self.event.clone())
    }
}

pub fn start_juggle(
    floor: &Floor,
    hit_id: EntityId,
    now: (u32, EntityId),
    stun_turns: NonZero<u32>,
) -> Writer<Entity, FloorEvent> {
    let coming_turn = now.0 + 1 + u32::from(hit_id < now.1);

    let state = match floor.entities[hit_id].state {
        EntityState::Ok { .. }
        | EntityState::Committed { .. }
        | EntityState::ConfirmCommand { .. }
        | EntityState::RestrictedActions { .. } => Writer::new(EntityState::Hitstun {
            next_round: coming_turn + stun_turns.get(),
            extensions: 1,
        })
        .log(FloorEvent::JuggleHit(JuggleHitEvent { target: hit_id })),

        EntityState::Hitstun { extensions, .. } => match extensions {
            0 => Writer::new(EntityState::Knockdown {
                next_round: coming_turn,
            })
            .log(FloorEvent::JuggleLimit(JuggleLimitEvent { target: hit_id })),
            nonzero => Writer::new(EntityState::Hitstun {
                next_round: coming_turn + 1,
                extensions: nonzero - 1,
            })
            .log(FloorEvent::JuggleHit(JuggleHitEvent { target: hit_id })),
        },
        EntityState::Knockdown { .. } => Writer::new(EntityState::Ok {
            next_round: coming_turn,
        }), // TODO: Log OTG hit.
        EntityState::Downed { .. } | EntityState::Exited { .. } => unreachable!(),
    };

    state.map(|state| Entity {
        state,
        ..floor.entities[hit_id].clone()
    })
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
        let floor = Floor::new_minimal();
        let (update, id) = floor.add_entity(Entity::default()).split_pair();

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
        let floor = Floor::new_minimal();
        let (update, id) = floor.add_entity(Entity::default()).split_pair();
        let (update, other) = update
            .bind(|floor| {
                floor.add_entity(Entity {
                    pos: AbsolutePosition::new(1, 0),
                    ..Entity::default()
                })
            })
            .split_pair();

        let update = update.bind(|floor| {
            TakeKnockbackUtil {
                entity: id,
                vector: RelativePosition::new(1, 0),
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
            EntityState::Knockdown { next_round: 0 }
        ));
        assert_eq!(
            update.get_log()[1],
            FloorEvent::KnockdownEvent(KnockdownEvent { subject: other })
        );
    }

    #[test]
    fn multiknockback_basic() {
        let floor = Floor::new_minimal();
        let (update, id) = floor.add_entity(Entity::default()).split_pair();
        let (update, other) = update
            .bind(|floor| {
                floor.add_entity(Entity {
                    pos: AbsolutePosition::new(1, 0),
                    ..Entity::default()
                })
            })
            .split_pair();

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
        let floor = Floor::new_minimal();
        let (update, id) = floor.add_entity(Entity::default()).split_pair();
        let (update, other) = update
            .bind(|floor| {
                floor.add_entity(Entity {
                    pos: AbsolutePosition::new(1, 0),
                    ..Entity::default()
                })
            })
            .split_pair();

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
            EntityState::Knockdown { next_round: 0 }
        ));
        assert_eq!(
            update.get_log()[1],
            FloorEvent::KnockdownEvent(KnockdownEvent { subject: other })
        );
    }

    #[test]
    fn multiknockback_conflicts() {
        let floor = Floor::new_minimal();
        let floor = floor.set_map(FloorMap::new_with_tiles(
            [
                (AbsolutePosition::new(0, 0), FloorTile::FLOOR),
                (AbsolutePosition::new(1, 0), FloorTile::FLOOR),
            ]
            .into(),
        ));

        let (update, id) = floor.add_entity(Entity::default()).split_pair();
        let (update, other) = update
            .bind(|floor| {
                floor.add_entity(Entity {
                    pos: AbsolutePosition::new(1, 0),
                    ..Entity::default()
                })
            })
            .split_pair();

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
            EntityState::Knockdown { next_round: 0 }
        ));
        assert_eq!(
            update.get_log()[2],
            FloorEvent::KnockdownEvent(KnockdownEvent { subject: other })
        );
    }
}
