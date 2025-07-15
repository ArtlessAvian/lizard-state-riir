use crate::direction::Direction;
use crate::walk::traits::IsAWalkPartial;

pub struct GenericWalkIterator<Walk: IsAWalkPartial> {
    inverse: Walk,
}

impl<Walk: IsAWalkPartial> GenericWalkIterator<Walk> {
    pub fn new(mut walk: Walk) -> Self {
        // HACK: We want a queue, but we have a stack interface.
        // Inverting both reverses the sequence and inverts each direction.
        // We'll need to revert the directions later.
        walk.inverse_mut();
        GenericWalkIterator { inverse: walk }
    }
}

impl<Walk: IsAWalkPartial> Iterator for GenericWalkIterator<Walk> {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        // HACK: We can pop directions the inverse of our path. We can revert individual directions now.
        self.inverse.pop_mut().ok().map(Direction::inverse)
    }
}
