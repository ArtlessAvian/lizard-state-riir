use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZero;
use std::ops::DerefMut;

use super::CommandTrait;
use super::events::FloorEvent;
use super::events::JuggleHitEvent;
use super::events::JuggleLimitEvent;
use super::events::KnockbackEvent;
use super::events::KnockdownEvent;
use super::known_serializable::KnownInfallibleAction;
use crate::entity::BatchEntityUpdate;
use crate::entity::Entity;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::RelativePosition;
use crate::positional::algorithms::Segment;
use crate::writer::Writer;

#[derive(Debug)]
pub struct TakeKnockbackUtil<'a> {
    pub parsed_floor: Cow<'a, Floor>,
    pub entity: EntityId,
    pub vector: RelativePosition,
}

impl CommandTrait for TakeKnockbackUtil<'_> {
    fn do_action(self) -> FloorUpdate {
        let now = self.parsed_floor.get_current_round();

        let swept_tiles = Segment::calculate_relative(self.vector)
            .0
            .into_iter()
            .map(|offset| self.parsed_floor.entities[self.entity].pos + offset)
            .take_while(|tile| self.parsed_floor.map.is_tile_floor(*tile))
            .collect::<Vec<_>>();

        let final_position = *swept_tiles
            .last()
            .expect("Entity's current position should be a floor, which is part of the segment");

        let mut writer = Writer::new(BatchEntityUpdate::new(&self.parsed_floor.entities));

        writer.deref_mut().apply_and_insert(self.entity, |e| {
            let mut clone = e.clone();
            clone.pos = final_position;
            clone
        });
        writer = writer.log({
            FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: self.entity,
                tile: final_position,
            })
        });

        let knocked_over = swept_tiles
            .iter()
            .filter_map(|tile| self.parsed_floor.occupiers.get(*tile))
            .filter(|id| *id != self.entity);

        for id in knocked_over {
            writer.apply_and_insert(id, |e| {
                let mut clone = e.clone();
                clone.state = EntityState::Knockdown { next_round: now };
                clone
            });
            writer = writer.log(FloorEvent::KnockdownEvent(KnockdownEvent { subject: id }));
        }

        writer.bind(|dingus| self.parsed_floor.update_entities_batch(dingus))
    }
}

#[derive(Debug)]
#[allow(dead_code, reason = "expect to use later")]
pub struct MultiKnockbackUtil<'a> {
    pub parsed_floor: Cow<'a, Floor>,
    pub all_displacements: HashMap<EntityId, RelativePosition>,
}

impl CommandTrait for MultiKnockbackUtil<'_> {
    fn do_action(self) -> FloorUpdate {
        let now = self.parsed_floor.get_current_round();

        let each_swept_tiles = self
            .all_displacements
            .iter()
            .map(|(id, displacement)| {
                (
                    *id,
                    Segment::calculate_relative(*displacement)
                        .0
                        .into_iter()
                        .map(|offset| self.parsed_floor.entities[*id].pos + offset)
                        .take_while(|tile| self.parsed_floor.map.is_tile_floor(*tile))
                        .collect::<Vec<_>>(),
                )
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

        let mut writer_batch = Writer::new(BatchEntityUpdate::new(&self.parsed_floor.entities));

        for (id, final_pos) in &final_positions {
            writer_batch.deref_mut().apply_and_insert(*id, |e| {
                let mut clone = e.clone();
                clone.pos = *final_pos;
                clone
            });
            writer_batch = writer_batch.log(FloorEvent::KnockbackEvent(KnockbackEvent {
                subject: *id,
                tile: *final_pos,
            }));
        }

        let conflict_knockdowns = final_positions
            .iter()
            .filter(|(id, pos)| {
                final_positions
                    .iter()
                    .filter(|(other_id, other_pos)| other_id != id && other_pos == pos)
                    .any(|(other_id, _)| {
                        let my_distance = self.parsed_floor.entities[*id].pos.distance(*pos);
                        let other_distance =
                            self.parsed_floor.entities[*other_id].pos.distance(*pos);
                        my_distance
                            .cmp(&other_distance)
                            .then(id.cmp(other_id).reverse())
                            .is_lt()
                        // We can ignore the eq case, since ids should be unique by design (and guaranteed by the previous filter)
                    })
            })
            .map(|(id, _)| *id);

        for id in conflict_knockdowns {
            writer_batch.deref_mut().apply_and_insert(id, |e| {
                let mut clone = e.clone();
                clone.state = EntityState::Knockdown { next_round: now };
                clone
            });
            writer_batch =
                writer_batch.log(FloorEvent::KnockdownEvent(KnockdownEvent { subject: id }));
        }

        let swept_tiles = each_swept_tiles
            .iter()
            .flat_map(|(_, segment)| segment.iter());

        let knocked_over = swept_tiles
            .filter_map(|tile| self.parsed_floor.occupiers.get(*tile))
            .filter(|id| self.all_displacements.iter().all(|(kb_id, _)| id != kb_id))
            .collect::<HashSet<_>>(); // Dedup!

        for id in knocked_over {
            writer_batch.deref_mut().apply_and_insert(id, |e| {
                let mut clone = e.clone();
                clone.state = EntityState::Knockdown { next_round: now };
                clone
            });
            writer_batch =
                writer_batch.log(FloorEvent::KnockdownEvent(KnockdownEvent { subject: id }));
        }

        writer_batch.bind(|batch| self.parsed_floor.update_entities_batch(batch))
    }
}

#[derive(Debug)]
pub struct DelayCommand<'a> {
    pub parsed_floor: Cow<'a, Floor>,
    pub subject_id: EntityId,
    pub queued_command: KnownInfallibleAction,
    pub turns: u32,
    pub event: Option<FloorEvent>,
}

impl CommandTrait for DelayCommand<'_> {
    fn do_action(self) -> FloorUpdate {
        let mut subject_clone: Entity = self.parsed_floor.entities[self.subject_id].clone();
        subject_clone.state = EntityState::Committed {
            next_round: self.parsed_floor.get_current_round() + self.turns,
            queued_command: self.queued_command.clone(),
        };

        self.parsed_floor
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
    use crate::floor::Floor;
    use crate::floor::map::FloorMap;
    use crate::floor::map::FloorTile;
    use crate::positional::AbsolutePosition;
    use crate::positional::RelativePosition;

    #[test]
    fn knockback_basic() {
        let floor = Floor::new_minimal();
        let (update, id) = floor.add_entity(Entity::default()).split_pair();

        let update = update.bind(|floor| {
            TakeKnockbackUtil {
                parsed_floor: Cow::Owned(floor),
                entity: id,
                vector: RelativePosition::new(1, 0),
            }
            .do_action()
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
                parsed_floor: Cow::Owned(floor),
                entity: id,
                vector: RelativePosition::new(0, 1),
            }
            .do_action()
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
                parsed_floor: Cow::Owned(floor),
                entity: id,
                vector: RelativePosition::new(1, 0),
            }
            .do_action()
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
                parsed_floor: Cow::Owned(floor),
                all_displacements: HashMap::from([
                    (id, RelativePosition::new(1, 0)),
                    (other, RelativePosition::new(1, 0)),
                ]),
            }
            .do_action()
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
        assert!(
            update
                .get_log()
                .contains(&FloorEvent::KnockbackEvent(KnockbackEvent {
                    subject: id,
                    tile: AbsolutePosition::new(1, 0)
                }))
        );
        assert!(
            update
                .get_log()
                .contains(&FloorEvent::KnockbackEvent(KnockbackEvent {
                    subject: other,
                    tile: AbsolutePosition::new(2, 0)
                }))
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
                parsed_floor: Cow::Owned(floor),
                all_displacements: HashMap::from([(id, RelativePosition::new(1, 0))]),
            }
            .do_action()
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
                (AbsolutePosition::new(0, 0), FloorTile::Floor),
                (AbsolutePosition::new(1, 0), FloorTile::Floor),
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
                parsed_floor: Cow::Owned(floor),
                all_displacements: HashMap::from([
                    (id, RelativePosition::new(1, 0)),
                    (other, RelativePosition::new(1, 0)),
                ]),
            }
            .do_action()
        });
        // TODO: Remove event ordering strictness.
        assert_eq!(
            update.get_contents().entities[id].pos,
            AbsolutePosition::new(1, 0)
        );
        assert!(
            update
                .get_log()
                .contains(&FloorEvent::KnockbackEvent(KnockbackEvent {
                    subject: id,
                    tile: AbsolutePosition::new(1, 0)
                }))
        );
        assert_eq!(
            update.get_contents().entities[other].pos,
            AbsolutePosition::new(1, 0)
        );
        assert!(
            update
                .get_log()
                .contains(&FloorEvent::KnockbackEvent(KnockbackEvent {
                    subject: other,
                    tile: AbsolutePosition::new(1, 0)
                }))
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
