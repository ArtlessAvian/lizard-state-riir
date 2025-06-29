use crate::direction::Direction;

pub mod edge_tiling;

/// Marker trait for tile types.
///
/// Enforces subtraits. Reduces typing.
pub trait IsATile: Copy + Eq {}

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
pub trait HasSquareTiling<Tile: IsATile>: Copy + Eq {
    /// Gets a consistent tile within the space.
    /// Implementor can panic if the space is empty.
    fn get_origin(&self) -> Tile;

    /// Given a `Tile` node in the graph, follow the `Direction` edge.
    ///
    /// May return `None`, bounding movement.
    ///
    /// Ensure that `step(t, dir) == Some(n)` if and only if `step(n, dir.inverse()) == Some(t)`
    fn step(&self, tile: &Tile, dir: Direction) -> Option<Tile>;

    /// Returns an iterator following the path, outputting every tile along the way (including the first).
    ///
    /// As an implementor, it doesn't make too much sense to override this.
    fn follow_path(
        &self,
        from: &Tile,
        path: impl IntoIterator<Item = Direction>,
    ) -> impl Iterator<Item = GoResult<Tile>> {
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
    fn skip_path(&self, from: &Tile, path: impl IntoIterator<Item = Direction>) -> GoResult<Tile> {
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
pub struct TileEdge<Tile> {
    pub tile: Tile,
    pub edge: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileUndirectedEdge<Tile> {
    Horizontal { left: Tile, right: Tile },
    Vertical { down: Tile, up: Tile },
}

impl<Tile> TileUndirectedEdge<Tile>
where
    Tile: IsATile,
{
    fn new<Space: HasSquareTiling<Tile>>(
        space: &Space,
        from: &Tile,
        dir: Direction,
    ) -> Option<Self> {
        let to = space.step(from, dir)?;

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
pub struct MissingEdgeError<Tile>(TileEdge<Tile>);

pub type GoResult<Tile> = Result<Tile, MissingEdgeError<Tile>>;
