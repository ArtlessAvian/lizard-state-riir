#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "type can grow maybe (and still be copy)"
)]

use crate::direction::Direction;

pub mod array_backed;
pub mod bits_backed;

pub trait PathLike
where
    Self: Default + Copy + Eq + IntoIterator<Item = Direction>,
{
    fn push(&self, dir: Direction) -> Option<Self>;

    fn pop(&self) -> Option<(Self, Direction)>;

    #[must_use]
    fn inverse(&self) -> Self {
        let mut out = Self::default();
        let mut copy = *self;
        while let Some((next, dir)) = copy.pop() {
            copy = next;
            out = out
                .push(dir.inverse())
                .expect("Assuming if PathLike implementer can represent a path of length N, it can represent all paths of length N");
        }
        out
    }

    fn extend(&self, path: impl IntoIterator<Item = Direction>) -> Option<Self> {
        path.into_iter().try_fold(*self, |out, dir| out.push(dir))
    }

    #[must_use]
    fn cancel_inverses(&self) -> Self {
        let mut out = Self::default();
        for dir in *self {
            if let Some((init, last)) = out.pop()
                && last.inverse().const_eq(dir)
            {
                out = init;
            } else {
                out = out.push(dir).expect("Assuming if PathLike implementer can represent a path of length N, it can represent all paths of length N");
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::direction::Direction;
    use crate::path::PathLike;
    use crate::path::array_backed::PathString;

    #[test]
    fn inverse() {
        let a = PathString::new_from_array([Direction::Up, Direction::Up, Direction::Right]);
        let a_inv = PathString::new_from_array([Direction::Left, Direction::Down, Direction::Down]);
        assert_eq!(a.inverse(), a_inv);
        assert_eq!(a.inverse().inverse(), a);
    }

    #[test]
    fn extend() {
        let empty: PathString<10> = PathString::new_empty();
        let a = PathString::new_from_array([Direction::Up, Direction::Up, Direction::Right]);

        let empty_a = empty.extend(a).unwrap();
        let empty_a_a = empty_a.extend(a).unwrap();
        let empty_a_a_a = empty_a_a.extend(a).unwrap();
        assert!(empty_a_a_a.extend(a).is_none());
    }

    #[test]
    fn cancel_inverses() {
        let empty: PathString<10> = PathString::new_empty();
        let a = PathString::new_from_array([Direction::Up, Direction::Up, Direction::Right]);
        let empty_a = empty.extend(a).unwrap();
        let empty_a_ainv = empty_a.extend(a.inverse()).unwrap();

        assert_eq!(empty_a_ainv.cancel_inverses(), empty);
    }
}
