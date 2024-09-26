use std::collections::HashMap;

use crate::entity::EntityId;
use crate::floor::map::FloorTile;
use crate::positional::AbsolutePosition;

/// A statement about something that happened in the game.
///
/// Not necessary to understand the state of the game, but rather what happened between states.
// TODO: Consider adding Rc<Entity> to events!
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FloorEvent {
    Move(MoveEvent),
    StartAttack(StartAttackEvent),
    AttackHit(AttackHitEvent),
    SeeMap(SeeMapEvent),
    KnockbackEvent(KnockbackEvent),
    KnockdownEvent(KnockdownEvent),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MoveEvent {
    pub subject: EntityId,
    pub tile: AbsolutePosition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StartAttackEvent {
    pub subject: EntityId,
    pub tile: AbsolutePosition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttackHitEvent {
    pub subject: EntityId,
    pub target: EntityId,
    pub damage: i32,
}

#[derive(PartialEq, Eq, Clone)]
pub struct SeeMapEvent {
    pub subject: EntityId,
    pub vision: HashMap<AbsolutePosition, FloorTile>,
}

impl std::fmt::Debug for SeeMapEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SeeMapEvent")
            .field("subject", &self.subject)
            // .field("vision", &self.vision)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct KnockbackEvent {
    pub subject: EntityId,
    pub tile: AbsolutePosition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct KnockdownEvent {
    pub subject: EntityId,
}
