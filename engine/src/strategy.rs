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
            RelativePosition::new(
                original.get_current_turn() as i32 % 3 - 1,
                (original.get_current_turn() as i32 + 1) % 3 - 1,
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