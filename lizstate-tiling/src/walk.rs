//! A sequence of edges, described by directions.
//! Without knowing the graph we are walking on, we cannot know if a walk is a path (no repeated vertices)
//! We can *sort of* know if a walk is a trail (no repeated edges).

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Walk contains no elements")]
pub struct WalkIsEmpty;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Walk cannot contain more elements")]
pub struct WalkIsFull;

// Yeah
pub mod traits;

// Implementations
pub mod direction_sequence;
#[cfg(feature = "std")]
pub mod vec;

// Wrappers
pub mod reduced;

#[cfg(all(test, feature = "std"))]
mod tests {
    use std::fmt::Debug;

    use crate::direction::Direction;
    use crate::walk::WalkIsEmpty;
    use crate::walk::traits::IsAWalkMut;

    fn pop_empty_err<Walk: IsAWalkMut>() {
        let mut walk = Walk::new_empty();
        assert_eq!(walk.pop_mut().unwrap_err(), WalkIsEmpty);
    }

    fn push_mut_eq_push_copy<Walk>()
    where
        Walk: IsAWalkMut + Copy + Debug,
    {
        let mut walk = Walk::new_empty();
        for _ in 0..10 {
            let res_copy = walk.push_copy(Direction::Right);
            let res_mut: Result<(), Walk::PushError> = walk.push_mut(Direction::Right);

            let copy = res_copy.unwrap();
            res_mut.unwrap();

            assert_eq!(walk, copy);
        }
    }

    // #[test]
    // fn walkenum_suite() {
    //     use crate::walk::enumeration::WalkEnum;

    //     pop_empty_err::<WalkEnum>();
    //     push_mut_eq_push_copy::<WalkEnum>();
    // }

    #[test]
    fn walk_enum_suite() {
        use crate::walk::direction_sequence::WalkEnum;

        pop_empty_err::<WalkEnum>();
        push_mut_eq_push_copy::<WalkEnum>();
    }

    #[test]
    fn reduced_walk_enum_suite() {
        use crate::walk::reduced::ReducedWalkEnum;

        pop_empty_err::<ReducedWalkEnum>();
        push_mut_eq_push_copy::<ReducedWalkEnum>();
    }

    #[test]
    fn walkvec_suite() {
        use crate::walk::vec::WalkVec;

        pop_empty_err::<WalkVec>();
    }
}
