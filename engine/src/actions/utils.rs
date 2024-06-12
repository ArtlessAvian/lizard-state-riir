use std::rc::Rc;

use crate::entity::EntityId;
use crate::floor::BorrowedFloorUpdate;
use crate::floor::Floor;
use crate::floor::FloorUpdate;
use crate::positional::RelativePosition;

use super::CommandTrait;

#[derive(Debug)]
pub struct TakeKnockbackUtil {
    pub entity: EntityId,
    pub vector: RelativePosition,
}

impl CommandTrait for TakeKnockbackUtil {
    fn do_action(&self, floor: &Floor) -> FloorUpdate {
        // intentionally incorrect, may panic due to invariant breaking lmao
        let writer = BorrowedFloorUpdate::new(floor);

        writer.bind(|f| {
            let mut updated = (*f.entities[self.entity]).clone();
            updated.pos = updated.pos + self.vector;
            f.update_entity(Rc::new(updated))
        })
    }
}
