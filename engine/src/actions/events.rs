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
    PrepareAttack(PrepareAttackEvent),
    StartAttack(StartAttackEvent),
    AttackHit(AttackHitEvent),
    JuggleHit(JuggleHitEvent),
    JuggleLimit(JuggleLimitEvent),
    SeeMap(SeeMapEvent),
    KnockbackEvent(KnockbackEvent),
    KnockdownEvent(KnockdownEvent),
    Wakeup(WakeupEvent),
    GetDowned(GetDownedEvent),
    Exit(ExitEvent),
    MissionFailed(MissionFailedEvent),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MoveEvent {
    pub subject: EntityId,
    pub tile: AbsolutePosition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrepareAttackEvent {
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JuggleHitEvent {
    pub target: EntityId,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JuggleLimitEvent {
    pub target: EntityId,
}

#[derive(PartialEq, Eq, Clone)]
pub struct SeeMapEvent {
    pub subject: EntityId,
    pub vision: HashMap<AbsolutePosition, FloorTile>,
    // Extra information, as a treat.
    // (helps marching squares look less surprising.)
    pub implied_tiles: HashMap<AbsolutePosition, FloorTile>,
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WakeupEvent {
    pub subject: EntityId,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GetDownedEvent {
    pub subject: EntityId,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExitEvent {
    pub subject: EntityId,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MissionFailedEvent {
    pub subject: EntityId,
    pub downed_party: Vec<EntityId>,
}
