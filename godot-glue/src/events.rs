use std::collections::HashMap;

use engine::actions::events::FloorEvent as FloorEventInternal;
use engine::entity::EntityId as EntityIdInternal;
use godot::prelude::*;

use crate::floor::EntityId;
use crate::positional::AbsolutePosition;

/// A statement about something that happened in the game.
///
/// Not necessary to understand the state of the game, but rather what happened between states.
// Some options, from strict to dynamic.
// # Wrapper for each case, store in VariantArray. Getter for each field.
// This preserves schema. Godot can do static analysis! But it will need to deduce the type first, eg InputEvent.
// Lots of glue code though. Maybe a macro can make getters. Or just expose the variable, the event is a throwaway value (mostly).
//
// # Wrapper around or convert from enum. Expose union of all fields wrapped in Option.
// Preserves some schema. Godot gets little type info.
// Godot will never key error (like a dict), but may read null values.
//
// # Convert to dictionary.
// No schema, no static analysis. Avoids repeated marshalling.
pub struct FloorEvent;

macro_rules! floor_event_to_variant {
    (($id_bijection:ident, $event:ident), $(($enum:ident, $eventty:ty),)*) => {
        match $event {
            $(
                FloorEventInternal::$enum(x) => <$eventty>::new($id_bijection, x).to_variant(),
            )*
        }
    };
}

impl FloorEvent {
    pub fn to_variant(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: FloorEventInternal,
    ) -> Variant {
        floor_event_to_variant!(
            (id_bijection, event),
            (Move, MoveEvent),
            (PrepareAttack, PrepareAttackEvent),
            (StartAttack, StartAttackEvent),
            (AttackHit, AttackHitEvent),
            (JuggleHit, JuggleHitEvent),
            (JuggleLimit, JuggleLimitEvent),
            (SeeMap, SeeMapEvent),
            (KnockbackEvent, KnockbackEvent),
            (KnockdownEvent, KnockdownEvent),
            (Wakeup, WakeupEvent),
        )
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct MoveEvent {
    #[var(get)]
    subject: Gd<EntityId>,
    #[var(get)]
    tile: AbsolutePosition,
}

impl MoveEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::MoveEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, id_bijection),
            tile: event.tile.into(),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct StartAttackEvent {
    #[var(get)]
    subject: Gd<EntityId>,
    #[var(get)]
    tile: AbsolutePosition,
}

impl StartAttackEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::StartAttackEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, id_bijection),
            tile: event.tile.into(),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct PrepareAttackEvent {
    #[var(get)]
    subject: Gd<EntityId>,
    #[var(get)]
    tile: AbsolutePosition,
}

impl PrepareAttackEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::PrepareAttackEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, id_bijection),
            tile: event.tile.into(),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct AttackHitEvent {
    #[var(get)]
    subject: Gd<EntityId>,
    #[var(get)]
    target: Gd<EntityId>,
    #[var(get)]
    damage: i32,
}

impl AttackHitEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::AttackHitEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, id_bijection),
            target: EntityId::new(event.target, id_bijection),
            damage: event.damage,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct JuggleHitEvent {
    #[var(get)]
    target: Gd<EntityId>,
}

impl JuggleHitEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::JuggleHitEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            target: EntityId::new(event.target, id_bijection),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct JuggleLimitEvent {
    #[var(get)]
    target: Gd<EntityId>,
}

impl JuggleLimitEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::JuggleLimitEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            target: EntityId::new(event.target, id_bijection),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct SeeMapEvent {
    #[var(get)]
    subject: Gd<EntityId>,
    #[var(get)]
    vision: Dictionary,
}

impl SeeMapEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::SeeMapEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, id_bijection),
            vision: event
                .vision
                .iter()
                .map(|(pos, tile)| {
                    (
                        AbsolutePosition::from(*pos),
                        *tile == engine::floor::map::FloorTile::FLOOR,
                    )
                })
                .collect(),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct KnockbackEvent {
    #[var(get)]
    subject: Gd<EntityId>,
    #[var(get)]
    tile: AbsolutePosition,
}

impl KnockbackEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::KnockbackEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, id_bijection),
            tile: event.tile.into(),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct KnockdownEvent {
    #[var(get)]
    subject: Gd<EntityId>,
}

impl KnockdownEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::KnockdownEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, id_bijection),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct WakeupEvent {
    #[var(get)]
    subject: Gd<EntityId>,
}

impl WakeupEvent {
    fn new(
        id_bijection: &mut HashMap<EntityIdInternal, Gd<EntityId>>,
        event: engine::actions::events::WakeupEvent,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, id_bijection),
        })
    }
}
