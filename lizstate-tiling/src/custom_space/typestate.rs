use crate::custom_space::CustomSpace;
use crate::custom_space::CustomSpaceTile;
use crate::direction::Direction;
use crate::tiling_graph::SpaceError;
use crate::tiling_graph::TileError;
use crate::walk::WalkIsFull;
use crate::walk::reduced::ReducedWalk;
use crate::walk::traits::IsAWalkPartial;

/// A known representative of a custom space.
/// The space becomes readonly.
#[must_use]
pub struct Representative<'a>(&'a CustomSpace, ReducedWalk);
impl<'a> Representative<'a> {
    fn new_origin(space: &'a CustomSpace) -> Self {
        Self(space, ReducedWalk::new_empty())
    }

    fn try_new_direct(space: &'a CustomSpace, path: &ReducedWalk) -> Option<Self> {
        if *path == ReducedWalk::new_empty() || space.contained_reps.contains(path) {
            Some(Self(space, *path))
        } else {
            None
        }
    }

    /// The entrypoint. Also see `CustomSpace::try_representative`.
    /// # Errors
    /// Space does not contain the (tile at the end of the path) / (an equivalent path to the given path).
    /// # Panics
    /// Space is in invalid state, value is not a representative.
    pub fn try_new_recursive(
        space: &'a CustomSpace,
        path: &ReducedWalk,
    ) -> Result<Representative<'a>, SpaceError> {
        if let Some(rep) = Representative::try_new_direct(space, path) {
            Ok(rep)
        } else {
            let (prefix, popped_dir) = path
                .pop_copy()
                .expect("tile does not eq the origin, which means its not empty");

            let prefix_rep = Self::try_new_recursive(space, &prefix)?;
            let rep = prefix_rep.try_step(popped_dir)?;
            Ok(rep)
        }
    }

    /// # Errors
    /// Space does not contain the step.
    /// # Panics
    /// Space is invalid, all values of `equivalent_rep` are representatives.
    pub fn try_step(self, dir: Direction) -> Result<Representative<'a>, SpaceError> {
        let stepped = self
            .1
            .push_copy(dir)
            .map_err(|WalkIsFull| SpaceError::NotInSpace)?;

        if let Some(rep) = Representative::try_new_direct(self.0, &stepped) {
            Ok(rep)
        } else if let Some(value) = self.0.equivalent_rep.get(&stepped) {
            let rep = Representative::try_new_direct(self.0, value)
                .expect("all values are representatives");
            Ok(rep)
        } else {
            Err(SpaceError::NotInSpace)
        }
    }

    pub fn step_toward_origin(self) -> Self {
        let (prefix, _) = self
            .1
            .pop_copy()
            .unwrap_or((ReducedWalk::new_empty(), Direction::Up));
        Self(self.0, prefix)
    }

    pub fn consume(self) -> CustomSpaceTile {
        CustomSpaceTile(self.1)
    }
}

#[must_use]
pub struct MutRepresentative<'a>(&'a mut CustomSpace, ReducedWalk);
impl<'a> MutRepresentative<'a> {
    fn new_origin(space: &'a mut CustomSpace) -> Self {
        Self(space, ReducedWalk::new_empty())
    }

    /// Adds tiles along the path. Returns the representative of the end of the path.
    /// # Errors
    /// `CustomSpace` cannot represent the tile at the end.
    pub fn insert_path_from_origin(
        space: &'a mut CustomSpace,
        path: ReducedWalk,
    ) -> Result<MutRepresentative<'a>, TileError> {
        let mut current = Self::new_origin(space);
        for dir in path {
            current = current.step_or_insert(dir)?;
        }
        Ok(current)
    }

    /// # Errors
    /// `CustomSpace` cannot represent the tile.
    pub fn step_or_insert(self, dir: Direction) -> Result<MutRepresentative<'a>, TileError> {
        let stepped = self
            .1
            .push_copy(dir)
            .map_err(|WalkIsFull| TileError::Unrepresentable)?;

        if self.0.contained_reps.contains(&stepped) {
            Ok(MutRepresentative(self.0, stepped))
        } else if let Some(rep) = self.0.equivalent_rep.get(&stepped) {
            Ok(MutRepresentative(self.0, *rep))
        } else {
            self.0.contained_reps.insert(stepped);
            Ok(MutRepresentative(self.0, stepped))
        }
    }

    pub fn step_toward_origin(self) -> Self {
        let (prefix, _) = self
            .1
            .pop_copy()
            .unwrap_or((ReducedWalk::new_empty(), Direction::Up));
        Self(self.0, prefix)
    }

    pub fn weaken(&'a self) -> Representative<'a> {
        Representative(self.0, self.1)
    }
}
