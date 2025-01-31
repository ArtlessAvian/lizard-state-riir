mod action;
mod actionset;

use std::rc::Rc;

use engine::actions::known_serializable::KnownUnaimedAction;
use engine::strategy::FollowStrategy;
use engine::strategy::NullStrategy;
use engine::strategy::RushdownStrategy;
use engine::strategy::StandAndFightStrategy;
use engine::strategy::Strategy;
use engine::strategy::WanderStrategy;
use godot::prelude::*;

use self::actionset::ActionSet;
use crate::floor::snapshot::EntitySnapshot;
use crate::positional::AbsolutePosition;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct EntityInitializer {
    base: Base<Resource>,
    #[export]
    position: AbsolutePosition,
    #[export]
    health: i8,
    #[export]
    max_energy: i8,
    /// Unforunate indirection.
    /// Nested resources are fine, but some tool is trying to read it?
    #[export]
    actions: GString,
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
        let moveset: Vec<KnownUnaimedAction> = if self.actions.is_empty() {
            Vec::new()
        } else {
            try_load::<ActionSet>(&self.actions)
                .map(|x| x.bind().to_vec())
                .unwrap_or_default()
        };

        engine::entity::Entity {
            state: engine::entity::EntityState::Ok { next_round: 0 },
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
    // TODO: Deprecate? We want to eventually have non unit struct strategies.
    fn to_actual(&self) -> Strategy {
        match self {
            StrategyName::Null => NullStrategy.into(),
            StrategyName::Wander => WanderStrategy.into(),
            StrategyName::StandAndFight => StandAndFightStrategy.into(),
            StrategyName::Follow => FollowStrategy.into(),
            StrategyName::Rushdown => RushdownStrategy.into(),
        }
    }
}
