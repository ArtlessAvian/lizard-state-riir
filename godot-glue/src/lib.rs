mod events;
mod positional;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use engine::actions::public::BumpAction;
use engine::actions::public::GotoAction;
use engine::actions::public::StepAction;
use engine::actions::public::StepMacroAction;
use engine::actions::ActionTrait;
use engine::actions::CommandTrait;
use engine::actions::DirectionActionTrait;
use engine::actions::TileActionTrait;
use engine::entity::Entity as EntityInternal;
use engine::entity::EntityId as EntityIdInternal;
use engine::entity::EntityState;
use engine::floor::Floor as FloorInternal;
use engine::positional::AbsolutePosition;
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

    // TODO: Sort of temporary. Maybe make a builder?
    // And a corresponding scene in Godot to match.
    #[func]
    pub fn set_map(&mut self, gridmap: Gd<godot::engine::GridMap>) {
        let tiles = gridmap
            .get_used_cells()
            .iter_shared()
            .filter(|vec| vec.y == 0)
            .map(|vec| {
                (
                    AbsolutePosition::new(vec.x, vec.z),
                    if gridmap.get_cell_item(vec) == 0 {
                        engine::floor::map::FloorTile::FLOOR
                    } else {
                        engine::floor::map::FloorTile::WALL
                    },
                )
            })
            .collect();

        let map = engine::floor::map::FloorMap {
            tiles: Rc::new(tiles),
            default: engine::floor::map::FloorTile::FLOOR,
        };

        self.floor = self.floor.set_map(map);
    }

    #[func]
    pub fn add_entity_at(&mut self, pos: Vector2i, is_player_controlled: bool) -> Gd<EntityId> {
        let (update, id) = self.floor.add_entity(EntityInternal {
            id: Default::default(),
            state: EntityState::Ok {
                next_turn: self.floor.get_current_turn(),
            },
            pos: AbsolutePosition::new(pos.x, pos.y),
            health: 10,
            is_player_controlled,
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
    pub fn get_step_action(&self) -> Gd<DirectionAction> {
        DirectionAction::new(Rc::new(StepAction))
    }

    #[func]
    pub fn get_bump_action(&self) -> Gd<DirectionAction> {
        DirectionAction::new(Rc::new(BumpAction))
    }

    #[func]
    pub fn get_step_macro_action(&self) -> Gd<DirectionAction> {
        DirectionAction::new(Rc::new(StepMacroAction))
    }

    #[func]
    pub fn get_goto_action(&self) -> Gd<TileAction> {
        TileAction::new(Rc::new(GotoAction))
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
}

/// An opaque object containing an Action. Has no logic.
#[derive(GodotClass)]
#[class(no_init)]
pub struct Action {
    // Godot doesn't see this anyways.
    action: Rc<dyn ActionTrait>,
}

#[godot_api]
impl Action {
    fn new(action: Rc<dyn ActionTrait>) -> Gd<Self> {
        Gd::from_object(Self { action })
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

/// An opaque object containing an Action. Has no logic.
#[derive(GodotClass)]
#[class(no_init)]
pub struct TileAction {
    // Godot doesn't see this anyways.
    action: Rc<dyn TileActionTrait>,
}

#[godot_api]
impl TileAction {
    fn new(action: Rc<dyn TileActionTrait>) -> Gd<Self> {
        Gd::from_object(Self { action })
    }

    #[func]
    fn to_command(
        &self,
        floor: Gd<Floor>,
        subject: Gd<Entity>,
        tile: crate::positional::AbsolutePosition,
    ) -> Option<Gd<Command>> {
        let binding = subject.bind();
        let subject_ref = &binding.entity;
        let verify_action =
            self.action
                .verify_action(&floor.bind().floor, subject_ref, tile.into())?;
        Some(Command::new(verify_action))
    }
}

/// An opaque object containing an Action. Has no logic.
#[derive(GodotClass)]
#[class(no_init)]
pub struct DirectionAction {
    // Godot doesn't see this anyways.
    action: Rc<dyn DirectionActionTrait>,
}

#[godot_api]
impl DirectionAction {
    fn new(action: Rc<dyn DirectionActionTrait>) -> Gd<Self> {
        Gd::from_object(Self { action })
    }

    #[func]
    fn to_command(
        &self,
        floor: Gd<Floor>,
        subject: Gd<Entity>,
        dir: crate::positional::RelativePosition,
    ) -> Option<Gd<Command>> {
        let binding = subject.bind();
        let subject_ref = &binding.entity;
        let verify_action =
            self.action
                .verify_action(&floor.bind().floor, subject_ref, dir.into())?;
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
