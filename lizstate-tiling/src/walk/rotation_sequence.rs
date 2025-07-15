use lizstate_sequence_enumeration::SequenceFull;
use lizstate_sequence_enumeration::SequenceOf;

use crate::direction::Direction;
use crate::direction::Nonbackwards;
use crate::direction::Rotation;
use crate::walk::WalkIsEmpty;
use crate::walk::WalkIsFull;
use crate::walk::generic_iter::GenericWalkIterator;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkPartial;
use crate::walk::traits::IsAWalkRaw;

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReducedWalkEnum {
    Nonempty(Nonempty),
    Empty,
}

impl ReducedWalkEnum {
    pub fn new_empty() -> Self {
        Self::Empty
    }

    pub fn new_with_initial(dir: Direction) -> Self {
        Self::Nonempty(Nonempty::new_with_initial(dir))
    }
}

// TODO: Pack initial direction and sequence into the same u64.
// Manual length management will be necessary.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nonempty(Direction, SequenceOf<Nonbackwards>);

impl Nonempty {
    fn new_with_initial(dir: Direction) -> Self {
        Self(dir, SequenceOf::new_empty())
    }

    fn len(&self) -> usize {
        1 + self.1.len()
    }

    fn peek(&self) -> Direction {
        let mut out = self.0;
        let mut copy = self.1;
        // Rotation application is commutative. We can just pop off the back.
        while let Ok(popped) = copy.pop() {
            out = Rotation::apply_to_direction(popped.into(), out);
        }
        out
    }
}

impl IsAWalkPartial for ReducedWalkEnum {
    type PushError = WalkIsFull;

    fn new_empty() -> Self
    where
        Self: Sized,
    {
        ReducedWalkEnum::Empty
    }

    fn len(&self) -> usize {
        match &self {
            ReducedWalkEnum::Nonempty(nonempty) => nonempty.len(),
            ReducedWalkEnum::Empty => 0,
        }
    }

    fn peek(&self) -> Result<Direction, WalkIsEmpty> {
        match self {
            ReducedWalkEnum::Nonempty(nonempty) => Ok(nonempty.peek()),
            ReducedWalkEnum::Empty => Err(WalkIsEmpty),
        }
    }

    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError> {
        match self {
            ReducedWalkEnum::Nonempty(nonempty) => {
                let peek = nonempty.peek();
                let rotation = Rotation::from_pair(peek, dir);
                if let Ok(nonbackwards) = Nonbackwards::try_from(rotation) {
                    nonempty
                        .1
                        .push(nonbackwards)
                        .map_err(|SequenceFull| WalkIsFull)?;
                } else {
                    self.prefix_mut();
                }
            }
            ReducedWalkEnum::Empty => {
                *self = Self::new_with_initial(dir);
            }
        }
        Ok(())
    }

    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty> {
        match self {
            ReducedWalkEnum::Nonempty(nonempty) => {
                let out = nonempty.peek();
                if nonempty.1.is_empty() {
                    *self = ReducedWalkEnum::Empty;
                } else {
                    _ = nonempty.1.pop();
                }
                Ok(out)
            }
            ReducedWalkEnum::Empty => Err(WalkIsEmpty),
        }
    }
}

// TODO: This probably sucks.
impl IntoIterator for ReducedWalkEnum {
    type Item = Direction;
    type IntoIter = GenericWalkIterator<Self>;

    fn into_iter(self) -> Self::IntoIter {
        GenericWalkIterator::new(self)
    }
}

impl IsAWalk for ReducedWalkEnum {}

impl IsAWalkRaw for ReducedWalkEnum {}
