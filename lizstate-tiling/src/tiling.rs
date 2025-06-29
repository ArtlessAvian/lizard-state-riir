use crate::direction::Direction;

pub mod edge_tiling;

/// Marker trait for tile types.
///
/// Enforces subtraits. Reduces typing.
pub trait Tile: Copy + Eq {}

/// Trait for spaces with a square tiling of `Tile`s.
///
/// `Space`s are high level descriptions of a 4-regular graph.
/// `Tile`s are the vertices of that graph, each with one outgoing edge for every `Direction`.
/// From neighboring nodes, there must be an incoming edge with the `Direction::inverse` direction.
///
/// When the `Space` is a unit type, the graph is known entirely at compile time.
///
/// # Correctness
/// Implementors must ensure that `step(t, dir) == Some(n)` if and only if `step(n, dir.inverse()) == Some(t)`.
///
/// An example of an invalid implementation is a graph with only `Up` and `Right` edges.
/// Another example is a two vertex graph, one vertex with four edges to the other, and the other having four self-loops.
pub trait HasSquareTiling<TileType: Tile>: Copy + Eq {
    /// Gets a consistent tile within the space.
    /// Implementor can panic if the space is empty.
    fn get_origin(&self) -> TileType;

    /// Given a `Tile` node in the graph, follow the `Direction` edge.
    ///
    /// May return `None`, bounding movement.
    ///
    /// Ensure that `step(t, dir) == Some(n)` if and only if `step(n, dir.inverse()) == Some(t)`
    fn step(&self, tile: &TileType, dir: Direction) -> Option<TileType>;

    /// Returns an iterator following the path, outputting every tile along the way (including the first).
    ///
    /// As an implementor, it doesn't make too much sense to override this.
    fn follow_path(
        &self,
        from: &TileType,
        path: impl IntoIterator<Item = Direction>,
    ) -> impl Iterator<Item = GoResult<TileType>> {
        let path = path.into_iter();

        let tail = path.scan(Ok(*from), |status, dir| {
            if let Ok(current) = status {
                *status = self.step(current, dir).ok_or(MissingEdgeError(TileEdge {
                    tile: *current,
                    edge: dir,
                }));
            }
            Some(*status)
        });

        [Ok(*from)].into_iter().chain(tail)
    }

    /// Gets the tile at the end of the path.
    ///
    /// # Errors
    /// When a step of the path is invalid. Returns the edge attempted.
    fn skip_path(
        &self,
        from: &TileType,
        path: impl IntoIterator<Item = Direction>,
    ) -> GoResult<TileType> {
        path.into_iter().try_fold(*from, |current, dir| {
            self.step(&current, dir).ok_or({
                MissingEdgeError(TileEdge {
                    tile: current,
                    edge: dir,
                })
            })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileEdge<TileType> {
    pub tile: TileType,
    pub edge: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileUndirectedEdge<TileType> {
    Horizontal { left: TileType, right: TileType },
    Vertical { down: TileType, up: TileType },
}

impl<TileType> TileUndirectedEdge<TileType>
where
    TileType: Tile,
{
    fn new<T: HasSquareTiling<TileType>>(
        tiling: &T,
        from: &TileType,
        dir: Direction,
    ) -> Option<Self> {
        let to = tiling.step(from, dir)?;

        Some(match dir {
            Direction::Up => TileUndirectedEdge::Vertical {
                down: *from,
                up: to,
            },
            Direction::Down => TileUndirectedEdge::Vertical {
                down: to,
                up: *from,
            },
            Direction::Right => TileUndirectedEdge::Horizontal {
                left: *from,
                right: to,
            },
            Direction::Left => TileUndirectedEdge::Horizontal {
                left: to,
                right: *from,
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingEdgeError<TileType>(TileEdge<TileType>);

pub type GoResult<TileType> = Result<TileType, MissingEdgeError<TileType>>;
