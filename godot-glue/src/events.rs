use engine::actions::events::FloorEvent as FloorEventInternal;

use godot::prelude::*;

use crate::positional::AbsolutePosition;
use crate::EntityId;
use crate::Floor;

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

impl FloorEvent {
    pub fn to_variant(floor: &mut Floor, event: FloorEventInternal) -> Variant {
        match event {
            FloorEventInternal::Move(x) => MoveEvent::new(floor, x).to_variant(),
            FloorEventInternal::StartAttack(x) => StartAttackEvent::new(floor, x).to_variant(),
            FloorEventInternal::AttackHit(x) => AttackHitEvent::new(floor, x).to_variant(),
            // default => Variant::nil(),
        }
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
    fn new(floor: &mut Floor, event: engine::actions::events::MoveEvent) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, &mut floor.id_bijection),
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
    fn new(floor: &mut Floor, event: engine::actions::events::StartAttackEvent) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, &mut floor.id_bijection),
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
    fn new(floor: &mut Floor, event: engine::actions::events::AttackHitEvent) -> Gd<Self> {
        Gd::from_object(Self {
            subject: EntityId::new(event.subject, &mut floor.id_bijection),
            target: EntityId::new(event.target, &mut floor.id_bijection),
            damage: event.damage,
        })
    }
}
