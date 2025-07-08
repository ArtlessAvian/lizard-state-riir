use crate::direction::Direction;

pub mod array_backed;
pub mod bits_backed;
#[cfg(feature = "std")]
pub mod vec_backed;

/// A path is efficient if there are no adjacent inverses.
pub mod efficient;

pub struct PathAlreadyEmpty;

/// An immutable value type, behaving like a Vec, but with bounded size.
///
/// Works in `no_std` contexts.
/// For longer paths, see `UnboundedPathLike`, which drops the `Copy` bound.
pub trait BoundedPathLike
where
    Self: Default + Clone + Eq + IntoIterator<Item = Direction>,
    Self: Copy,
{
    const MAX_CAPACITY: usize;

    fn push(&self, dir: Direction) -> Option<Self>;

    /// # Errors
    /// The path is already empty.
    fn pop(&self) -> Result<(Self, Direction), PathAlreadyEmpty>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the path backwards with inverse directions.
    ///
    /// This undoes the path.
    /// `path.extend(path.inverse()) == Path::new_empty()`
    #[must_use]
    fn inverse(&self) -> Self {
        let mut out = Self::default();
        let mut copy = *self;
        while let Ok((next, dir)) = copy.pop() {
            copy = next;
            out = out
                .push(dir.inverse())
                .expect("Assuming if PathLike implementer can represent a path of length N, it can represent all paths of length N");
        }
        out
    }

    /// Appends one path to another.
    ///
    /// May return None if path cannot be represented by the implementer.
    fn extend(&self, path: impl IntoIterator<Item = Direction>) -> Option<Self> {
        path.into_iter().try_fold(*self, |out, dir| out.push(dir))
    }

    /// Cancels if the last direction is opposite, otherwise pushes.
    #[must_use]
    fn push_or_cancel(&self, dir: Direction) -> Option<Self> {
        if let Ok((init, last)) = self.pop()
            && last.inverse() == dir
        {
            Some(init)
        } else {
            self.push(dir)
        }
    }

    /// Cleans up redundant steps in the path.
    #[must_use]
    fn cancel_inverses(&self) -> Self {
        let mut out = Self::default();
        for dir in *self {
            out = out.push_or_cancel(dir).expect("Assuming if PathLike implementer can represent a path of length N, it can represent all paths of length N");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::direction::Direction;
    use crate::path::BoundedPathLike;
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
