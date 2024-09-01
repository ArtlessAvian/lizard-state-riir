use std::rc::Rc;

use engine::entity::Entity as EntityInternal;
use godot::prelude::*;

use crate::actions::Action;
use crate::actions::Command;
use crate::actions::DirectionAction;
use crate::actions::TileAction;

/// A snapshot of an Entity. Has no logic.
///
/// Does not update when the Floor updates.
/// Only contains getters. It is impossible to have setters.
#[derive(GodotClass)]
#[class(no_init)]
pub struct Entity {
    pub entity: Rc<EntityInternal>,
}

#[godot_api]
impl Entity {
    pub fn new(entity: Rc<EntityInternal>) -> Gd<Self> {
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
            })
            .collect()
    }

    #[func]
    fn get_command_to_confirm(&self) -> Option<Gd<Command>> {
        self.entity.get_command_to_confirm().map(Command::new)
    }

    #[func]
    fn get_debug(&self) -> String {
        format!("{:?}", self.entity.state)
    }
}
