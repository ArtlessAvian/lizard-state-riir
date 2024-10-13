mod action;
mod actionset;

use std::rc::Rc;

use engine::strategy::FollowStrategy;
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
    #[export]
    actions: Option<Gd<ActionSet>>,
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
        let moveset = self
            .actions
            .as_ref()
            .map(|x| x.bind().to_vec())
            .unwrap_or_default();

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

// #[godot_api]
// impl IResource for EntityInitializer {
//     fn set_property(&mut self, property: StringName, value: Variant) -> bool {
//         godot_print!("HI IM IN HERE");
//         if property == "actions".into() {
//             if value.is_nil() {
//                 self.actions = None;
//                 return true;
//             }
//             if let Ok(unwrap) = value.try_to::<Gd<ActionSet>>() {
//                 self.actions = Some(unwrap);
//                 return true;
//             }
//             return false;
//         }
//         return false;
//     }
// }

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
