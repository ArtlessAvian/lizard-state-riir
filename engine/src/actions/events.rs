use crate::entity::EntityId;
use crate::positional::AbsolutePosition;

/// A statement about something that happened in the game.
///
/// Not necessary to understand the state of the game, but rather what happened between states.
#[derive(Debug, PartialEq, Eq)]
pub enum FloorEvent {
    Move(MoveEvent),
    StartAttack(StartAttackEvent),
    AttackHit(AttackHitEvent),
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