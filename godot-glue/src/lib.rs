mod events;
mod positional;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
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
use events::FloorEvent;
use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

/// The game.
///
/// Container for the Floor and an accumulated log.
/// From Godot, you can read the log, read game info, get actions, submit actions.
/// Other than that, there should be no game logic, eg information hiding.
///
/// Note that FloorInternal is entirely pure functions, but this wrapper does not the same.
#[derive(GodotClass)]
#[class(init)]
pub struct Floor {
    floor: FloorInternal,
    #[export]
    log: VariantArray,
    id_bijection: HashMap<EntityIdInternal, Gd<EntityId>>,
}

#[godot_api]
impl Floor {
    /// Since Floor (in glue code) is not a pure class unlike the Floor (in engine),
    /// this is here to explicitly copy.
    #[func]
    pub fn duplicate(&self) -> Gd<Floor> {
        Gd::from_object(Floor {
            floor: self.floor.clone(),
            log: self.log.duplicate_shallow(),
            id_bijection: self.id_bijection.clone(),
        })
    }

    #[func]
    pub fn add_entity_at(&mut self, pos: Vector2i) -> Gd<EntityId> {
        let (update, id) = self.floor.add_entity(EntityInternal {
            id: Default::default(),
            next_turn: Some(0),
            pos: AbsolutePosition::new(pos.x, pos.y),
            health: 10,
        });

        let (next, log) = update.into_both();
        self.floor = next;

        let temp = log
            .into_iter()
            .map(|ev| FloorEvent::to_variant(self, ev))
            .collect();
        self.log.extend_array(temp);

        EntityId::new(id, &mut self.id_bijection)
    }

    #[func]
    pub fn get_entity_ids(&mut self) -> Array<Gd<EntityId>> {
        self.floor
            .entities
            .iter()
            .map(|e| e.id)
            .map(|e| EntityId::new(e, &mut self.id_bijection))
            .collect()
    }

    #[func]
    pub fn get_entity_by_id(&self, id: Gd<EntityId>) -> Gd<Entity> {
        Entity::new(Rc::clone(&self.floor.entities[id.bind().id]))
    }

    #[func]
    pub fn take_npc_turn(&mut self) {
        // TODO: handle err.
        let result = self.floor.take_npc_turn();
        if let Ok(update) = result {
            let (next, log) = update.into_both();
            self.floor = next;
            let temp = log
                .into_iter()
                .map(|ev| FloorEvent::to_variant(self, ev))
                .collect();
            self.log.extend_array(temp);
        }
    }

    #[func]
    pub fn do_action(&mut self, command: Gd<Command>) {
        let (next, log) = command.bind().command.do_action(&self.floor).into_both();
        self.floor = next;

        let temp = log
            .into_iter()
            .map(|ev| FloorEvent::to_variant(self, ev))
            .collect();
        self.log.extend_array(temp)
    }

    // engine::actions::public::* goes here.

    #[func]
    pub fn get_step_action(&self, direction: Vector2i) -> Gd<Action> {
        Action::new(Box::new(StepAction {
            dir: RelativePosition::new(direction.x, direction.y),
        }))
    }

    #[func]
    pub fn get_bump_action(&self, direction: Vector2i) -> Gd<Action> {
        Action::new(Box::new(BumpAction {
            dir: RelativePosition::new(direction.x, direction.y),
        }))
    }

    #[func]
    pub fn get_step_macro_action(&self, direction: Vector2i) -> Gd<Action> {
        Action::new(Box::new(StepMacroAction {
            dir: RelativePosition::new(direction.x, direction.y),
        }))
    }
}

/// An opaque EntityId.
///
/// An alternative was to map to an i32 for Godot to avoid allocation.
/// This lets Godot try to convert random i32s *back* to internal ids, which is error prone.
/// Or encourages implementation detail shenanigans, like comparison or arithmetic.
#[derive(GodotClass)]
#[class(no_init)]
pub struct EntityId {
    id: EntityIdInternal,
    #[var(get)]
    petname: GString,
    _use_constructor: (),
}

impl EntityId {
    // Every id has one internal by definition.
    // The hashmap ensures every internal has one id.
    // So its a bijection.
    pub fn new(
        id: EntityIdInternal,
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
    ) -> Gd<Self> {
        match id_bijection.entry(id) {
            Entry::Occupied(el) => el.get().clone(),
            Entry::Vacant(slot) => slot
                .insert(Gd::from_object(EntityId {
                    id,
                    petname: petname::Petnames::default().generate_one(2, "-").into(),
                    _use_constructor: (),
                }))
                .clone(),
        }
    }
}

#[godot_api]
impl IRefCounted for EntityId {
    fn to_string(&self) -> GString {
        self.petname.clone()
    }
}

/// A snapshot of an Entity. Has no logic.
///
/// Does not update when the Floor updates.
/// Only contains getters. It is impossible to have setters.
#[derive(GodotClass)]
#[class(no_init)]
pub struct Entity {
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

/// An opaque object containing an Action. Has no logic.
#[derive(GodotClass)]
pub struct Action {
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

/// An opaque object representing a Command. Has no logic.
///
/// Note the inversion between object and param compared to Engine (though that may change).
/// ```rust
/// // Note this will not run as a doctest since godot_glue is a cdylib.
/// use engine::data::Floor as FloorInternal;
/// use engine::actions::Command as CommandInternal;
/// use godot_glue::Floor;
/// use godot_glue::Command;
///
/// fn engine_context(floor: &FloorInternal, command: &Box<dyn CommandTrait>) {
///     command.do_action(floor);
/// }
/// fn glue_context(floor: &mut Floor, command: &Command) {
///     floor.do_action(command);
/// }
/// ```
#[derive(GodotClass)]
#[class(no_init)]
pub struct Command {
    // Godot doesn't see this anyways.
    command: Box<dyn CommandTrait>,
}

#[godot_api]
impl Command {
    fn new(command: Box<dyn CommandTrait>) -> Gd<Self> {
        Gd::from_object(Self { command })
    }
}
