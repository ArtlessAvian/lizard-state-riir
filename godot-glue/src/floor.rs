pub(crate) mod snapshot;

use std::collections::HashMap;
use std::default::Default;
use std::rc::Rc;

use engine::entity::Entity as EntityInternal;
use engine::entity::EntityId as EntityIdInternal;
use engine::entity::EntityState;
use engine::floor::Floor as FloorInternal;
use engine::floor::FloorEndState;
use engine::positional::AbsolutePosition;
use engine::strategy::FollowStrategy;
use engine::strategy::Strategy;
use godot::prelude::*;
use tracing::instrument;

use crate::actions::Command;
use crate::events::FloorEvent;
use crate::floor::snapshot::EntitySnapshot;
use crate::resources::EntityInitializer;

#[derive(GodotConvert, Var, Export)]
#[godot(via = GString)]
#[derive(Debug)]
pub enum FloorEndStateName {
    Undetermined,
    AnyPartyMemberDowned,
    AllPartyMembersExited,
    ExitedButNoFood,
}

impl From<FloorEndState> for FloorEndStateName {
    fn from(value: FloorEndState) -> Self {
        match value {
            FloorEndState::Undetermined => FloorEndStateName::Undetermined,
            FloorEndState::AnyPartyMemberDowned => FloorEndStateName::AnyPartyMemberDowned,
            FloorEndState::AllPartyMembersExited => FloorEndStateName::AllPartyMembersExited,
            FloorEndState::ExitedButNoFood => FloorEndStateName::ExitedButNoFood,
        }
    }
}

/// The game.
///
/// Container for the Floor and an accumulated log.
/// From Godot, you can read the log, read game info, get actions, submit actions.
/// Other than that, there should be no game logic, eg information hiding.
///
/// Note that `FloorInternal` is entirely pure functions, but this wrapper does not the same.
// TODO: Add FloorSnapshot?
#[derive(GodotClass)]
#[class(init)]
pub struct ActiveFloor {
    pub internal: FloorInternal,
    #[export]
    log: VariantArray,
    pub id_cache: EntityIdCache,
}

#[godot_api]
impl ActiveFloor {
    /// Since Floor (in glue code) is not a pure class unlike the Floor (in engine),
    /// this is here to explicitly copy.
    #[func]
    #[must_use]
    pub fn duplicate(&self) -> Gd<ActiveFloor> {
        Gd::from_object(ActiveFloor {
            internal: self.internal.clone(),
            log: self.log.duplicate_shallow(),
            id_cache: self.id_cache.clone(),
        })
    }

    // TODO: Sort of temporary. Maybe make a builder?
    // And a corresponding scene in Godot to match.
    #[func]
    pub fn set_map(&mut self, gridmap: Gd<godot::classes::GridMap>) {
        let tiles = gridmap
            .get_used_cells()
            .iter_shared()
            .filter(|vec| vec.y == 0)
            .map(|vec| {
                (
                    AbsolutePosition::new(vec.x, vec.z),
                    if gridmap.get_cell_item(vec) == 0 {
                        engine::floor::map::FloorTile::Floor
                    } else {
                        engine::floor::map::FloorTile::Wall
                    },
                )
            })
            .collect();

        let map = engine::floor::map::FloorMap::new_with_tiles(tiles);

        self.internal = self.internal.set_map(map);
    }

    // TODO: Sort of temporary. Maybe make a builder?
    // And a corresponding scene in Godot to match.
    #[func]
    pub fn set_map_2d(&mut self, tilemap: Gd<godot::classes::TileMap>) {
        let tiles = tilemap
            .get_used_cells(0)
            .iter_shared()
            .map(|vec| {
                (
                    AbsolutePosition::new(vec.x, vec.y),
                    engine::floor::map::FloorTile::Floor,
                )
            })
            .collect();

        let map = engine::floor::map::FloorMap::new_with_tiles(tiles);

        self.internal = self.internal.set_map(map);
    }

    #[func]
    pub fn add_entity_at(
        &mut self,
        pos: Vector2i,
        is_player_controlled: bool,
        is_player_friendly: bool,
    ) -> Gd<EntityId> {
        let (update, id) = self
            .internal
            .add_entity(EntityInternal {
                state: EntityState::Ok {
                    next_round: self.internal.get_current_round(),
                },
                pos: AbsolutePosition::new(pos.x, pos.y),
                health: 10,
                max_energy: 6,
                energy: 6,
                moveset: Vec::new(),
                strategy: Strategy::Follow(FollowStrategy {}),
                is_player_controlled,
                is_player_friendly,
                payload: String::default(),
            })
            .split_pair();

        let (next, log) = update.into_both();
        self.internal = next;

        let temp = log
            .into_iter()
            .map(|ev| FloorEvent::to_variant(&mut self.id_cache, ev))
            .collect();
        self.log.extend_array(&temp);

        self.id_cache.get_or_insert(id)
    }

    #[func]
    pub fn add_entity_from_initializer(
        &mut self,
        initializer: Gd<EntityInitializer>,
    ) -> Gd<EntityId> {
        let (update, id) = self
            .internal
            .add_entity(initializer.bind().to_entity())
            .split_pair();

        let (next, log) = update.into_both();
        self.internal = next;

        let temp = log
            .into_iter()
            .map(|ev| FloorEvent::to_variant(&mut self.id_cache, ev))
            .collect();
        self.log.extend_array(&temp);

        self.id_cache.get_or_insert(id)
    }

    #[func]
    pub fn get_entity_ids(&mut self) -> Array<Gd<EntityId>> {
        self.internal
            .entities
            .iter_ids()
            .map(|e| self.id_cache.get_or_insert(e))
            .collect()
    }

    #[func]
    #[must_use]
    pub fn get_entity_by_id(&self, id: Gd<EntityId>) -> Gd<EntitySnapshot> {
        EntitySnapshot::new(Rc::clone(self.internal.entities.index_as_rc(id.bind().id)))
    }

    #[func]
    pub fn take_npc_turn(&mut self) -> bool {
        // TODO: handle err.
        let result = self.internal.take_npc_turn();
        if let Ok(update) = result {
            let (next, log) = update.into_both();
            self.internal = next;
            let temp = log
                .into_iter()
                .map(|ev| FloorEvent::to_variant(&mut self.id_cache, ev))
                .collect();
            self.log.extend_array(&temp);

            return true;
        }
        false
    }

    // TODO: Use Godot built in error types!
    #[func]
    #[instrument(skip_all)]
    pub fn do_action(&mut self, mut command: Gd<Command>) -> bool {
        let inner = command.bind_mut().command.take();
        if let Some(some) = inner {
            let (next, log) = some.do_action().into_both();
            self.internal = next;

            let temp = log
                .into_iter()
                .map(|ev| FloorEvent::to_variant(&mut self.id_cache, ev))
                .collect();
            self.log.extend_array(&temp);

            return true;
        }
        false
    }

    #[func]
    #[must_use]
    pub fn get_time(&self) -> u32 {
        self.internal.get_current_round()
    }

    #[func]
    #[must_use]
    pub fn get_end_state(&self) -> FloorEndStateName {
        self.internal.get_end_state().into()
    }
}

/// An opaque `EntityId`.
///
/// An alternative was to map to an i32 for Godot to avoid allocation.
/// This lets Godot try to convert random i32s *back* to internal ids, which is error prone.
/// Or encourages implementation detail shenanigans, like comparison or arithmetic.
#[derive(GodotClass)]
#[class(no_init)]
pub struct EntityId {
    pub id: EntityIdInternal,
    #[var(get)]
    petname: GString,
    _use_constructor: (),
}

#[derive(Clone, Default)]
pub struct EntityIdCache(HashMap<EntityIdInternal, Gd<EntityId>>);

impl EntityIdCache {
    pub fn get_or_insert(&mut self, id: EntityIdInternal) -> Gd<EntityId> {
        self.0
            .entry(id)
            .or_insert_with(|| {
                Gd::from_object(EntityId {
                    id,
                    petname: petname::Petnames::default().generate_one(2, "-").into(),
                    _use_constructor: (),
                })
            })
            .clone()
    }
}

#[godot_api]
impl IRefCounted for EntityId {
    fn to_string(&self) -> GString {
        self.petname.clone()
    }
}
