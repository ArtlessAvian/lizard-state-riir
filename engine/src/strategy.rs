use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

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
pub enum Strategy {
    Null,
    Wander(WanderStrategy),
    StandAndFight(StandAndFightStrategy),
    Follow(FollowStrategy),
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
        let in_range = original
            .entities
            .iter_entities()
            .find(|x| x.pos.distance(original.entities[subject_id].pos) <= 2 && x.id != subject_id);

        if let Some(other) = in_range {
            let unaimed_action = &original.entities[subject_id].get_actions()[3];
            let lmao = match unaimed_action {
                crate::actions::UnaimedAction::Tile(hi_this_is_temporary) => {
                    hi_this_is_temporary.verify_action(original, subject_id, other.pos)
                }
                _ => None,
            };
            if let Some(command) = lmao {
                return command.do_action(original);
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
pub struct FollowStrategy;

impl StrategyTrait for FollowStrategy {
    fn take_turn(&self, original: &Floor, subject_id: EntityId) -> FloorUpdate {
        // TODO: Add teams/friendliness to the game.
        let in_range = original
            .entities
            .iter_entities()
            .find(|x| x.pos.distance(original.entities[subject_id].pos) <= 6 && x.id != subject_id);

        let subject = &original.entities[subject_id];

        if let Some(other) = in_range {
            if subject.pos.distance(other.pos) > 2 {
                if let Some(step_to) = original.map.get_step(subject.pos, other.pos) {
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
