use super::WalkIsEmpty;
use crate::direction::Direction;
use crate::walk::direction_sequence::WalkEnum;
use crate::walk::traits::IsAWalkMut;
use crate::walk::traits::IsAWalkRaw;

// "Reduced" like a "word" from group theory.
#[must_use]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reduced<Walk: IsAWalkRaw>(Walk);

impl<Walk: IsAWalkRaw> IsAWalkMut for Reduced<Walk> {
    type PushError = Walk::PushError;

    fn new_empty() -> Self {
        Reduced(Walk::new_empty())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn peek(&self) -> Result<Direction, WalkIsEmpty> {
        self.0.peek()
    }

    fn push_mut(&mut self, dir: Direction) -> Result<(), Self::PushError> {
        if self
            .peek()
            .is_ok_and(|peeked| peeked.const_eq(dir.inverse()))
        {
            let _ = self.0.pop_mut();
            Ok(())
        } else {
            self.0.push_mut(dir)
        }
    }

    fn pop_mut(&mut self) -> Result<Direction, WalkIsEmpty> {
        self.0.pop_mut()
    }
}

impl<Walk> IntoIterator for Reduced<Walk>
where
    Walk: IsAWalkRaw,
    Walk: IntoIterator<Item = Direction>,
{
    type Item = Direction;
    type IntoIter = Walk::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub type ReducedWalkEnum = Reduced<WalkEnum>;

#[cfg(test)]
mod tests {
    use crate::direction::Direction;
    use crate::walk::reduced::ReducedWalkEnum;
    use crate::walk::traits::IsAWalkMut;

    #[test]
    fn string_append() {
        let empty = ReducedWalkEnum::new_empty();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let right = empty.push_copy(Direction::Right).unwrap();
        assert!(!right.is_empty());
        assert_eq!(right.len(), 1);

        let right_left = right.push_copy(Direction::Left).unwrap();
        assert!(right_left.is_empty());
        assert_eq!(right_left.len(), 0);

        let right_left_left = right_left.push_copy(Direction::Left).unwrap();
        assert!(!right_left_left.is_empty());
        assert_eq!(right_left_left.len(), 1);
    }

    #[test]
    fn string_iter() {
        let path = [
            Direction::Right,
            Direction::Right,
            Direction::Left,
            Direction::Down,
            Direction::Down,
        ];

        let string = path
            .into_iter()
            .try_fold(ReducedWalkEnum::new_empty(), |string, dir| {
                ReducedWalkEnum::push_copy(&string, dir)
            })
            .unwrap();

        let mut iter = string.into_iter();
        for expected in [Direction::Right, Direction::Down, Direction::Down] {
            assert_eq!(iter.next().unwrap(), expected);
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn string_overflow() {
        let mut string = ReducedWalkEnum::new_empty();

        for _ in 0..31 {
            string = string.push_copy(Direction::Right).unwrap();
        }

        assert!(string.push_copy(Direction::Right).is_err());
    }
}
