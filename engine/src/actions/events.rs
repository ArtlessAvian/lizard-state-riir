use std::collections::HashMap;

use crate::entity::EntityId;
use crate::floor::map::FloorTile;
use crate::positional::AbsolutePosition;

/// A statement about something that happened in the game.
///
/// Not necessary to understand the state of the game, but rather what happened between states.
#[derive(Debug, PartialEq, Eq)]
pub enum FloorEvent {
    Move(MoveEvent),
    StartAttack(StartAttackEvent),
    AttackHit(AttackHitEvent),
    SeeMap(SeeMapEvent),
    KnockbackEvent(KnockbackEvent),
}

#[derive(Debug, PartialEq, Eq)]
pub struct MoveEvent {
    pub subject: EntityId,
    pub tile: AbsolutePosition,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StartAttackEvent {
    pub subject: EntityId,
    pub tile: AbsolutePosition,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AttackHitEvent {
    pub subject: EntityId,
    pub target: EntityId,
    pub damage: i32,
}

#[derive(PartialEq, Eq)]
pub struct SeeMapEvent {
    pub subject: EntityId,
    pub vision: HashMap<AbsolutePosition, FloorTile>,
}

impl std::fmt::Debug for SeeMapEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SeeMapEvent")
            .field("subject", &self.subject)
            // .field("vision", &self.vision)
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct KnockbackEvent {
    pub subject: EntityId,
    pub tile: AbsolutePosition,
}
