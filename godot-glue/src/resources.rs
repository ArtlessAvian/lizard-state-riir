use std::rc::Rc;

use engine::actions::SerializableUnaimedAction;
use engine::strategy::FollowStrategy;
use engine::strategy::RushdownStrategy;
use engine::strategy::StandAndFightStrategy;
use engine::strategy::Strategy;
use engine::strategy::WanderStrategy;
use godot::prelude::*;

use crate::floor::snapshot::EntitySnapshot;
use crate::positional::AbsolutePosition;

#[derive(GodotClass, Debug)]
#[class(tool, init, base=Resource)]
pub struct EntityInitializer {
    base: Base<Resource>,
    #[export]
    position: AbsolutePosition,
    #[export]
    health: i8,
    #[export]
    max_energy: i8,
    #[export]
    strategy: StrategyName,
    #[export]
    is_player_controlled: bool,
    #[export]
    is_player_friendly: bool,
    #[export]
    passthrough: GString,
}

#[godot_api]
impl EntityInitializer {
    #[must_use]
    pub fn to_entity(&self) -> engine::entity::Entity {
        let moveset: Vec<SerializableUnaimedAction> = Vec::new();
        // moveset.push(SerializableAction::Direction(Rc::new(DoubleHitAction {})));
        // moveset.push(SerializableAction::None(Rc::new(EnterStanceAction {})));
        // moveset.push(SerializableAction::Direction(Rc::new(
        //     ForwardHeavyAction {},
        // )));
        // moveset.push(SerializableAction::Tile(Rc::new(TrackingAction {})));

        engine::entity::Entity {
            state: engine::entity::EntityState::Ok { next_turn: 0 },
            pos: self.position.into(),
            health: self.health,
            max_energy: self.max_energy,
            energy: self.max_energy,
            // TODO: Configure through resource.
            moveset,
            strategy: self.strategy.to_actual(),
            is_player_controlled: self.is_player_controlled,
            is_player_friendly: self.is_player_friendly,
            // Use the payload to hold "non-model" or Godot related info.
            payload: self.passthrough.to_string(),
        }
    }

    #[func]
    #[must_use]
    pub fn to_snapshot(&self) -> Gd<EntitySnapshot> {
        Gd::from_object(EntitySnapshot {
            entity: Rc::new(self.to_entity()),
        })
    }
}

#[derive(GodotConvert, Var, Export, Default)]
#[godot(via = GString)]
#[derive(Debug)]
pub enum StrategyName {
    #[default]
    Null,
    Wander,
    StandAndFight,
    Follow,
    Rushdown,
}

impl StrategyName {
    fn to_actual(&self) -> Strategy {
        match self {
            StrategyName::Null => Strategy::Null,
            StrategyName::Wander => Strategy::Wander(WanderStrategy),
            StrategyName::StandAndFight => Strategy::StandAndFight(StandAndFightStrategy),
            StrategyName::Follow => Strategy::Follow(FollowStrategy),
            StrategyName::Rushdown => Strategy::Rushdown(RushdownStrategy),
        }
    }
}
