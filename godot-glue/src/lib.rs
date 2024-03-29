use std::rc::Rc;

use engine::actions::public::BumpAction;
use engine::actions::public::StepAction;
use engine::actions::public::StepMacroAction;
use engine::actions::ActionTrait;
use engine::actions::CommandTrait;
use engine::actions::NullAction;
use engine::data::Entity as EntityInternal;
use engine::data::Floor as FloorInternal;
use engine::positional::AbsolutePosition;
use engine::positional::RelativePosition;
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
    fn add_entity(&mut self) -> i32 {
        self.floor = self.floor.add_entity(Rc::new(EntityInternal {
            pos: AbsolutePosition { x: 0, y: 0 },
            health: 10,
        }));

        (self.floor.entities.len() - 1).try_into().unwrap()
    }

    #[func]
    fn add_entity_at(&mut self, pos: Vector2i) -> i32 {
        self.floor = self.floor.add_entity(Rc::new(EntityInternal {
            pos: AbsolutePosition { x: pos.x, y: pos.y },
            health: 10,
        }));

        (self.floor.entities.len() - 1).try_into().unwrap()
    }

    #[func]
    fn get_player(&self) -> Gd<Entity> {
        Entity::new(self.floor.get_player())
    }

    #[func]
    fn get_entity_by_id(&self, id: i32) -> Gd<Entity> {
        let thing: usize = id.try_into().unwrap();
        Entity::new(Rc::clone(self.floor.entities.get(thing).unwrap()))
    }

    // engine::actions::public::* goes here.

    #[func]
    fn get_step_action(&self, direction: Vector2i) -> Gd<Action> {
        Action::new(Box::new(StepAction {
            dir: RelativePosition {
                dx: direction.x,
                dy: direction.y,
            },
        }))
    }

    #[func]
    fn get_bump_action(&self, direction: Vector2i) -> Gd<Action> {
        Action::new(Box::new(BumpAction {
            dir: RelativePosition {
                dx: direction.x,
                dy: direction.y,
            },
        }))
    }

    #[func]
    fn get_step_macro_action(&self, direction: Vector2i) -> Gd<Action> {
        Action::new(Box::new(StepMacroAction {
            dir: RelativePosition {
                dx: direction.x,
                dy: direction.y,
            },
        }))
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

    #[func]
    fn get_pos(&self) -> Vector2i {
        Vector2i {
            x: self.entity.as_ref().unwrap().pos.x,
            y: self.entity.as_ref().unwrap().pos.y,
        }
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
            action: Box::new(NullAction {}),
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
