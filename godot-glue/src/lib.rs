use std::rc::Rc;

use engine::actions::GoRightAction;
use engine::data::ActionTrait;
use engine::data::CommandTrait;
use engine::data::Entity;
use engine::data::Floor as FloorInternal;
use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
struct Floor {
    floor: FloorInternal,
}

#[godot_api]
impl IRefCounted for Floor {
    fn init(_base: Base<RefCounted>) -> Self {
        Self {
            floor: FloorInternal::new(),
        }
    }
}

#[godot_api]
impl Floor {
    #[func]
    fn add_entity(&mut self) {
        self.floor = self.floor.add_entity(Rc::new(Entity { x: 3 }));
    }

    #[func]
    fn go_right(&mut self) {
        self.floor = GoRightAction
            .verify_action(&self.floor, &self.floor.get_player())
            .unwrap()
            .do_action(&self.floor);
    }
}
