use std::rc::Rc;

use engine::actions::example::GoRightAction;
use engine::actions::ActionTrait;
use engine::actions::CommandTrait;
use engine::data::Entity as EntityInternal;
use engine::data::Floor as FloorInternal;
use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(init)]
struct Floor {
    floor: FloorInternal,
}

#[godot_api]
impl Floor {
    #[func]
    fn add_entity(&mut self) {
        self.floor = self.floor.add_entity(Rc::new(EntityInternal { x: 3 }));
    }

    #[func]
    fn go_right(&mut self) {
        self.floor = GoRightAction
            .verify_action(&self.floor, &self.floor.get_player())
            .unwrap()
            .do_action(&self.floor);
    }

    #[func]
    fn get_player(&self) -> Gd<Entity> {
        Entity::new(self.floor.get_player())
    }

    #[func]
    fn get_action(&self) -> Gd<Action> {
        Action::new(Box::new(GoRightAction))
    }
}

#[derive(GodotClass)]
struct Entity {
    entity: Option<Rc<EntityInternal>>,
}

#[godot_api]
impl IRefCounted for Entity {
    fn init(_base: Base<RefCounted>) -> Self {
        Self { entity: None }
    }
}

#[godot_api]
impl Entity {
    fn new(entity: Rc<EntityInternal>) -> Gd<Self> {
        Gd::from_init_fn(|_base| Self {
            entity: Some(entity),
        })
    }
}

#[derive(GodotClass)]
struct Action {
    // Godot doesn't see this anyways.
    action: Box<dyn ActionTrait>,
}

#[godot_api]
impl IRefCounted for Action {
    // Return a null action.
    fn init(_base: Base<RefCounted>) -> Self {
        Self {
            action: Box::new(GoRightAction),
        }
    }
}

#[godot_api]
impl Action {
    fn new(action: Box<dyn ActionTrait>) -> Gd<Self> {
        Gd::from_init_fn(|_base| Self { action })
    }

    #[func]
    fn to_command(&self, floor: Gd<Floor>, subject: Gd<Entity>) -> Option<Gd<Command>> {
        let binding = subject.bind();
        let subject_ref = binding.entity.as_ref()?;
        let verify_action = self
            .action
            .verify_action(&floor.bind().floor, subject_ref)?;
        Some(Command::new(verify_action))
    }
}

#[derive(GodotClass)]
struct Command {
    // Godot doesn't see this anyways.
    command: Option<Box<dyn CommandTrait>>,
}

#[godot_api]
impl IRefCounted for Command {
    // Return an invalid command. Don't call this.
    fn init(_base: Base<RefCounted>) -> Self {
        Self { command: None }
    }
}

#[godot_api]
impl Command {
    fn new(command: Box<dyn CommandTrait>) -> Gd<Self> {
        Gd::from_init_fn(|_base| Self {
            command: Some(command),
        })
    }

    #[func]
    fn do_action(&self, floor: Gd<Floor>) -> Gd<Floor> {
        let bind = Gd::bind(&floor);
        let next: FloorInternal = self.command.as_ref().unwrap().do_action(&bind.floor);
        Gd::from_object(Floor { floor: next })
    }
}
