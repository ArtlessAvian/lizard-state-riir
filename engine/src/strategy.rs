use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::actions::public::BumpAction;
use crate::actions::public::StepAction;
use crate::actions::public::WaitAction;
use crate::actions::ActionTrait;
use crate::actions::DirectionActionTrait;
use crate::entity::EntityId;
use crate::entity::EntityState;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::RelativePosition;

pub trait StrategyTrait {
    fn take_turn(&self, original: &Floor, subject_id: EntityId) -> FloorUpdate;
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[cfg_attr(test, derive(Default))]
pub enum Strategy {
    #[cfg_attr(test, default)]
    Null,
    Wander(WanderStrategy),
    StandAndFight(StandAndFightStrategy),
    Follow(FollowStrategy),
    Rushdown(RushdownStrategy),
}

impl StrategyTrait for Strategy {
    #[must_use]
    fn take_turn(&self, original: &Floor, subject_id: EntityId) -> FloorUpdate {
        match self {
            Strategy::Null => WaitAction {}
                .verify_action(original, subject_id)
                .expect("Wait should never fail")
                .do_action(original),
            Strategy::Wander(x) => x.take_turn(original, subject_id),
            Strategy::StandAndFight(x) => x.take_turn(original, subject_id),
            Strategy::Follow(x) => x.take_turn(original, subject_id),
            Strategy::Rushdown(x) => x.take_turn(original, subject_id),
        }
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct WanderStrategy;

impl StrategyTrait for WanderStrategy {
    fn take_turn(&self, original: &Floor, subject_id: EntityId) -> FloorUpdate {
        if let Some(x) = StepAction.verify_action(
            original,
            subject_id,
            #[allow(clippy::cast_possible_wrap, clippy::unreadable_literal)]
            RelativePosition::new(
                ((original.get_current_round() ^ 0xfedcba) % 3) as i32 - 1,
                ((original.get_current_round() ^ 0xabcdef) % 3) as i32 - 1,
            ),
        ) {
            return x.do_action(original);
        }

        WaitAction {}
            .verify_action(original, subject_id)
            .expect("Wait should never fail")
            .do_action(original)
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct StandAndFightStrategy;

impl StrategyTrait for StandAndFightStrategy {
    fn take_turn(&self, original: &Floor, subject_id: EntityId) -> FloorUpdate {
        // let in_range = original.entities.iter().find(|(id, entity)| {
        //     entity.pos.distance(original.entities[subject_id].pos) <= 2 && *id != subject_id
        // });

        // if let Some(other) = in_range {
        //     let unaimed_action = &original.entities[subject_id].get_actions()[3];
        //     let lmao = match unaimed_action {
        //         crate::actions::UnaimedAction::Tile(hi_this_is_temporary) => {
        //             hi_this_is_temporary.verify_action(original, subject_id, other.1.pos)
        //         }
        //         _ => None,
        //     };
        //     if let Some(command) = lmao {
        //         return command.do_action(original);
        //     }
        // }

        WaitAction {}
            .verify_action(original, subject_id)
            .expect("Wait should never fail")
            .do_action(original)
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct FollowStrategy;

impl StrategyTrait for FollowStrategy {
    fn take_turn(&self, original: &Floor, subject_id: EntityId) -> FloorUpdate {
        // TODO: Add teams/friendliness to the game.
        let subject = &original.entities[subject_id];

        let in_range = original
            .entities
            .iter()
            .filter(|(id, entity)| entity.pos.distance(subject.pos) <= 6 && *id != subject_id)
            .collect::<Vec<_>>();

        let ally_in_range = in_range
            .iter()
            .find(|(_id, entity)| entity.is_allied(subject));

        if let Some((_, ally)) = ally_in_range {
            // avoid standing on top of the ally
            if subject.pos.distance(ally.pos) == 0 {
                for dir in [
                    RelativePosition::new(1, 0),
                    RelativePosition::new(0, 1),
                    RelativePosition::new(-1, 0),
                    RelativePosition::new(0, -1),
                    RelativePosition::new(1, 1),
                    RelativePosition::new(1, -1),
                    RelativePosition::new(-1, 1),
                    RelativePosition::new(-1, -1),
                ] {
                    if let Some(x) = StepAction.verify_action(original, subject_id, dir) {
                        return x.do_action(original);
                    }
                }
            }
        }

        if let Some((_, ally)) = ally_in_range {
            // If ally is too far, stepping is priority
            if subject.pos.distance(ally.pos) > 4 {
                if let Some(step_to) = original.map.get_step(subject.pos, ally.pos) {
                    if let Some(x) =
                        StepAction.verify_action(original, subject_id, step_to - subject.pos)
                    {
                        return x.do_action(original);
                    }
                }
            }
        }

        if let Some((_, enemy)) = in_range
            .iter()
            .find(|(_, entity)| !entity.is_allied(subject) && entity.pos.distance(subject.pos) == 1)
        {
            if matches!(enemy.state, EntityState::Knockdown { .. }) {
                if let Some(x) =
                    StepAction.verify_action(original, subject_id, -(enemy.pos - subject.pos))
                {
                    return x.do_action(original);
                }
            } else if let Some(x) =
                BumpAction.verify_action(original, subject_id, enemy.pos - subject.pos)
            {
                return x.do_action(original);
            }
        }

        if let Some((_ally_id, ally)) = ally_in_range {
            if subject.pos.distance(ally.pos) > 2 {
                if let Some(step_to) = original.map.get_step(subject.pos, ally.pos) {
                    if let Some(x) =
                        StepAction.verify_action(original, subject_id, step_to - subject.pos)
                    {
                        return x.do_action(original);
                    }
                }
            }
        }

        WaitAction {}
            .verify_action(original, subject_id)
            .expect("Wait should never fail")
            .do_action(original)
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct RushdownStrategy;

impl StrategyTrait for RushdownStrategy {
    fn take_turn(&self, original: &Floor, subject_id: EntityId) -> FloorUpdate {
        let subject = &original.entities[subject_id];

        let in_range = original
            .entities
            .iter()
            .filter(|(id, entity)| *id != subject_id && !subject.is_allied(entity))
            .min_by_key(|(_, entity)| entity.pos.distance(subject.pos))
            .filter(|(_, entity)| entity.pos.distance(subject.pos) <= 6);

        if let Some(other) = in_range {
            if subject.pos.distance(other.1.pos) > 1 {
                if let Some(step_to) = original.map.get_step(subject.pos, other.1.pos) {
                    if let Some(x) =
                        StepAction.verify_action(original, subject_id, step_to - subject.pos)
                    {
                        return x.do_action(original);
                    }
                }
            } else if let Some(x) =
                BumpAction.verify_action(original, subject_id, other.1.pos - subject.pos)
            {
                return x.do_action(original);
            }
        }

        WaitAction {}
            .verify_action(original, subject_id)
            .expect("Wait should never fail")
            .do_action(original)
    }
}
