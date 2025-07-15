use thiserror::Error;

use crate::direction::Direction;
use crate::walk::reduced::Reduced;
use crate::walk::traits::IsAWalk;
use crate::walk::traits::IsAWalkPartial;
use crate::walk::traits::IsAWalkRaw;

/// Marker trait for tile/vertex types.
///
/// Enforces subtraits. Reduces typing.
/// You can imagine these as indexes or keys into a space.
/// Like keys, you can't expect to have a valid key in one map and have it be valid in another map.
/// So be careful with mutable spaces.
///
/// Two tiles that are `Eq` represent the same tile in *every* space.
pub trait IsATile: Copy + Eq {}

/// Marker trait for space/graph types.
pub trait IsASpace {}

#[derive(Debug, Error)]
pub enum TileError {
    #[error("Tile cannot be reasonably represented in memory")]
    Unrepresentable,
}

#[derive(Debug, Error)]
pub enum SpaceError {
    #[error("Space does not contain tile")]
    NotInSpace,
}

#[derive(Debug, Error)]
pub enum StepError {
    #[error("Space does not contain argument")]
    ArgumentNotInSpace,
    #[error("Destination cannot be reasonably represented in memory")]
    DestinationUnrepresentable,
    #[error("Space does not contain destination")]
    DestinationNotInSpace,
}

/// 4-regular undirected graphs. There is one outgoing directed edges for every `Direction`.
///
/// When the implementor is a unit type, the graph is known entirely at compile time.
///
/// # Correctness
/// Implementors must ensure that `step(t, dir) == Some(n)` if and only if `step(n, dir.inverse()) == Some(t)`.
///
/// An example of an invalid implementation is a graph with only `Up` and `Right` edges.
/// Another example is a two vertex graph, one vertex with four edges to the other, and the other having four self-loops.
pub trait IsTilingGraph: IsASpace {
    /// A reasonable way to think about the vertices in the graph.
    /// Remember that two tiles that are `Eq` represent the same location in *every* space.
    type Tile: IsATile;

    /// Gets a consistent tile within the space. Any tile can be the origin.
    /// Most of the time, it's `Default::default()`, but don't be so sure.
    fn get_origin(&self) -> Self::Tile;

    /// Given a `Tile` vertex in the graph, follow the `Direction` edge.
    ///
    /// Ensure that `step(t, dir) == Some(n)` if and only if `step(n, dir.inverse()) == Some(t)`
    /// # Errors
    /// Implementor may return if the space it represents does not contain either end of the step,
    /// or if the Tile cannot be represented.
    fn step(&self, tile: &Self::Tile, dir: Direction) -> Result<Self::Tile, StepError>;
}

/// Trait for **efficiently** converting paths to tiles.
///
/// This should always be implementable.
/// Default implementations are linear time.
/// Feel free to overwrite them if a better one exists.
///
/// Also a desirable trait of the place you live. /j
pub trait IsWalkable: IsTilingGraph {
    /// Walks from a tile and returns the destination.
    ///
    /// Default impl takes time linear to the length of the walk.
    /// # Errors
    /// Any step is invalid.
    fn walk_from_tile(
        &self,
        from: &Self::Tile,
        walk: impl IntoIterator<Item = Direction>,
    ) -> Result<Self::Tile, StepError> {
        walk.into_iter()
            .try_fold(*from, |current, dir| self.step(&current, dir))
    }

    /// Walks from the origin and returns the destination.
    ///
    /// Default impl takes time linear to the length of the walk.
    /// # Errors
    /// Any step is invalid.
    fn walk_from_origin(
        &self,
        walk: impl IntoIterator<Item = Direction>,
    ) -> Result<Self::Tile, StepError> {
        self.walk_from_tile(&self.get_origin(), walk)
    }
}

// pub struct Path(Walk);

/// Trait for **efficiently** converting tiles to paths.
///
/// Returned paths are not shortest, they just need to be connected.
///
/// Default implementations time linear to the output.
/// Do not do worse than that, like an O(V^2) search!
pub trait CanFindArbitraryPath: IsTilingGraph {
    /// # Errors
    /// Path is too long to be represented by `Walk`
    fn path_from_origin<Walk: IsAWalkRaw>(
        &self,
        to: &Self::Tile,
    ) -> Result<Reduced<Walk>, Walk::PushError>;

    /// # Errors
    /// Path is too long to be represented by `Walk`
    fn path_to_origin<Walk: IsAWalkRaw>(
        &self,
        from: &Self::Tile,
    ) -> Result<Reduced<Walk>, Walk::PushError> {
        let mut out = self.path_from_origin::<Walk>(from)?;
        out.inverse_mut();
        Ok(out)
    }

    /// # Errors
    /// Path is too long to be represented by `Walk`
    fn path_between_tiles<Walk: IsAWalkRaw>(
        &self,
        from: &Self::Tile,
        to: &Self::Tile,
    ) -> Result<Reduced<Walk>, Walk::PushError> {
        let out = self.path_to_origin::<Walk>(from)?;
        let extension = self.path_from_origin::<Walk>(to)?;

        out.try_extend(extension)
    }
}
