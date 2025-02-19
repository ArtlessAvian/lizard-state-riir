use std::rc::Rc;

use engine::entity::Entity;
use engine::entity::EntityState;
use godot::prelude::*;

use crate::actions::Action;
use crate::actions::DirectionAction;
use crate::actions::InfallibleAction;
use crate::actions::TileAction;

/// Does not update when the Floor updates.
/// Only contains getters. It is impossible to have setters.
#[derive(GodotClass)]
#[class(no_init)]
pub struct EntitySnapshot {
    pub entity: Rc<Entity>,
}

#[godot_api]
impl EntitySnapshot {
    #[must_use]
    pub fn new(entity: Rc<Entity>) -> Gd<Self> {
        Gd::from_object(Self { entity })
    }

    #[func]
    fn get_pos(&self) -> Vector2i {
        Vector2i::new(self.entity.pos.x, self.entity.pos.y)
    }

    #[func]
    fn get_health(&self) -> i8 {
        self.entity.health
    }

    #[func]
    fn get_energy(&self) -> i8 {
        self.entity.energy
    }

    #[func]
    fn get_actions(&self) -> VariantArray {
        self.entity
            .get_actions()
            .iter()
            .map(|unaimed| match unaimed {
                engine::actions::UnaimedAction::None(x) => Action::new(Rc::clone(x)).to_variant(),
                engine::actions::UnaimedAction::Tile(x) => {
                    TileAction::new(Rc::clone(x)).to_variant()
                }
                engine::actions::UnaimedAction::Direction(x) => {
                    DirectionAction::new(Rc::clone(x)).to_variant()
                }
                engine::actions::UnaimedAction::Infallible(x) => {
                    InfallibleAction::new(Rc::clone(x)).to_variant()
                }
            })
            .collect()
    }

    #[func]
    fn get_command_to_confirm(&self) -> Option<Gd<InfallibleAction>> {
        Some(InfallibleAction::new(Rc::new(
            self.entity.get_command_to_confirm()?,
        )))
    }

    #[func]
    fn get_debug(&self) -> String {
        format!("{:?}", self.entity.state)
    }

    #[func]
    fn get_state_name(&self) -> EntityStateName {
        match self.entity.state {
            EntityState::Ok { .. } => EntityStateName::Ok,
            EntityState::Committed { .. } => EntityStateName::Committed,
            EntityState::ConfirmCommand { .. } => EntityStateName::ConfirmCommand,
            EntityState::RestrictedActions { .. } => EntityStateName::RestrictedActions,
            EntityState::Hitstun { .. } => EntityStateName::Hitstun,
            EntityState::Knockdown { .. } => EntityStateName::Knockdown,
            EntityState::Downed { .. } => EntityStateName::Downed,
            EntityState::Exited { .. } => EntityStateName::Exited,
        }
    }

    // Passthrough as defined by the EntityInitializer.
    // If not constructed by EntityInitializer, this is whatever string.
    #[func]
    fn get_passthrough(&self) -> GString {
        self.entity.get_payload().into()
    }
}

#[derive(GodotConvert, Var, Export)]
#[godot(via = GString)]
#[derive(Debug)]
pub enum EntityStateName {
    Ok,
    Committed,
    ConfirmCommand,
    RestrictedActions,
    Hitstun,
    Knockdown,
    Downed,
    Exited,
}
