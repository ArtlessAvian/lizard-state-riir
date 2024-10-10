use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::actions::public::BumpAction;
use crate::actions::public::StepAction;
use crate::actions::public::WaitAction;
use crate::actions::ActionTrait;
use crate::actions::DirectionActionTrait;
use crate::entity::EntityId;
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
                ((original.get_current_turn() ^ 0xfedcba) % 3) as i32 - 1,
                ((original.get_current_turn() ^ 0xabcdef) % 3) as i32 - 1,
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
        let in_range = original.entities.iter().find(|(id, entity)| {
            entity.pos.distance(original.entities[subject_id].pos) <= 6 && *id != subject_id
        });

        let subject = &original.entities[subject_id];

        if let Some(other) = in_range {
            if subject.pos.distance(other.1.pos) > 2 {
                if let Some(step_to) = original.map.get_step(subject.pos, other.1.pos) {
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

        let in_range = original.entities.iter().find(|(id, entity)| {
            entity.pos.distance(subject.pos) <= 6 && *id != subject_id && !subject.is_allied(entity)
        });

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
