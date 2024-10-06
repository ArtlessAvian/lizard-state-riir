use std::rc::Rc;

use engine::strategy::FollowStrategy;
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
        engine::entity::Entity {
            state: engine::entity::EntityState::Ok { next_turn: 0 },
            pos: self.position.into(),
            health: self.health,
            max_energy: self.max_energy,
            energy: self.max_energy,
            strategy: engine::strategy::Strategy::Follow(FollowStrategy),
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
