use std::rc::Rc;

use engine::actions::public::BumpAction;
use engine::actions::public::StepAction;
use engine::actions::public::StepMacroAction;
use engine::actions::ActionTrait;
use engine::actions::CommandTrait;
use engine::actions::NullAction;
use engine::data::Floor as FloorInternal;
use engine::entity::Entity as EntityInternal;
use engine::entity::EntityId as EntityIdInternal;
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
    #[export]
    log: Array<i32>,
}

/// Container for the Floor and an accumulated log.
#[godot_api]
impl Floor {
    /// Since Floor (in glue code) is not a pure class unlike the Floor (in engine),
    /// this is here to explicitly copy.
    #[func]
    fn duplicate(&self) -> Gd<Floor> {
        Gd::from_object(Floor {
            floor: self.floor.clone(),
            log: self.log.duplicate_shallow(),
        })
    }

    #[func]
    fn add_entity_at(&mut self, pos: Vector2i) -> Gd<EntityId> {
        let id;
        (self.floor, id) = self.floor.add_entity(EntityInternal {
            id: Default::default(),
            next_turn: Some(0),
            pos: AbsolutePosition::new(pos.x, pos.y),
            health: 10,
        });
        Gd::from_object(EntityId { id })
    }

    #[func]
    fn get_entity_by_id(&self, id: Gd<EntityId>) -> Gd<Entity> {
        Entity::new(Rc::clone(&self.floor.entities[id.bind().id]))
    }

    #[func]
    fn take_npc_turn(&mut self) {
        // TODO: handle err.
        let result = self.floor.take_npc_turn();
        if let Ok((next, log)) = result {
            self.floor = next;
            let temp = log.iter().map(|_| 1).collect();
            self.log.extend_array(temp);
        }
    }

    #[func]
    fn do_action(&mut self, command: Gd<Command>) {
        let (next, log) = command.bind().command.do_action(&self.floor);
        self.floor = next;

        let temp = log.iter().map(|_| 1).collect();
        self.log.extend_array(temp)
    }

    // engine::actions::public::* goes here.

    #[func]
    fn get_step_action(&self, direction: Vector2i) -> Gd<Action> {
        Action::new(Box::new(StepAction {
            dir: RelativePosition::new(direction.x, direction.y),
        }))
    }

    #[func]
    fn get_bump_action(&self, direction: Vector2i) -> Gd<Action> {
        Action::new(Box::new(BumpAction {
            dir: RelativePosition::new(direction.x, direction.y),
        }))
    }

    #[func]
    fn get_step_macro_action(&self, direction: Vector2i) -> Gd<Action> {
        Action::new(Box::new(StepMacroAction {
            dir: RelativePosition::new(direction.x, direction.y),
        }))
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityId {
    id: EntityIdInternal,
}

#[derive(GodotClass)]
#[class(no_init)]
struct Entity {
    entity: Rc<EntityInternal>,
}

#[godot_api]
impl Entity {
    fn new(entity: Rc<EntityInternal>) -> Gd<Self> {
        Gd::from_object(Self { entity })
    }

    #[func]
    fn get_pos(&self) -> Vector2i {
        Vector2i::new(self.entity.pos.x, self.entity.pos.y)
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
        let subject_ref = &binding.entity;
        let verify_action = self
            .action
            .verify_action(&floor.bind().floor, subject_ref)?;
        Some(Command::new(verify_action))
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct Command {
    // Godot doesn't see this anyways.
    command: Box<dyn CommandTrait>,
}

#[godot_api]
impl Command {
    fn new(command: Box<dyn CommandTrait>) -> Gd<Self> {
        Gd::from_object(Self { command })
    }
}
