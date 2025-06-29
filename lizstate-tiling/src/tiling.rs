/// Marker trait for tile types.
///
/// Enforces subtraits. Reduces typing.
pub trait Tile: Copy + Eq {}

/// A tile's direct relation to their direct neighbor.
///
/// Each square tile has four neighbors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    pub const fn inverse(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }

    #[must_use]
    pub const fn const_eq(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Up)
                | (Direction::Down, Direction::Down)
                | (Direction::Right, Direction::Right)
                | (Direction::Left, Direction::Left)
        )
    }
}

/// Trait for spaces with a square tiling of `Tile`s.
///
/// `Space`s are high level descriptions of a 4-regular graph.
/// `Tile`s are the vertices of that graph, each with one outgoing edge for every `Direction`.
/// From neighboring nodes, there must be an incoming edge with the `Direction::inverse` direction.
///
/// When the `Space` is a unit type, the graph is known entirely at compile time.
///
/// # Correctness
/// Implementors must ensure that `go(t, dir) == Some(n)` if and only if `go(n, dir.inverse()) == Some(t)`.
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
    /// Ensure that `go(t, dir) == Some(n)` if and only if `go(n, dir.inverse()) == Some(t)`
    fn go(&self, tile: &TileType, dir: Direction) -> Option<TileType>;
}

////////////////////////////////////////////////////////////

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
        let to = tiling.go(from, dir)?;

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

impl<TileType> Tile for TileUndirectedEdge<TileType> where TileType: Tile {}

////////////////////////////////////////////////////////////

/// For every square in a tiling, replace every square's edge with a diamond.
///
/// Alternatively, replace every graph edge with a vertex.
/// Directions are rotated by 45 degrees.
#[derive(PartialEq, Eq)]
pub struct EdgeTiling<Tiling>(Tiling);

impl<Tiling, TileType> HasSquareTiling<TileUndirectedEdge<TileType>> for &EdgeTiling<Tiling>
where
    Tiling: HasSquareTiling<TileType>,
    TileType: Tile,
{
    fn get_origin(&self) -> TileUndirectedEdge<TileType> {
        let origin = self.0.get_origin();
        for dir in [
            Direction::Up,
            Direction::Down,
            Direction::Right,
            Direction::Left,
        ] {
            if let Some(edge) = TileUndirectedEdge::new(&self.0, &origin, dir) {
                return edge;
            }
        }
        panic!("origin is disconnected?")
    }

    fn go(
        &self,
        tile: &TileUndirectedEdge<TileType>,
        dir: Direction,
    ) -> Option<TileUndirectedEdge<TileType>> {
        match tile {
            // A horizontal edge's neighbor will always be vertical.
            //      O   O
            // LEFT |   | UP
            //      O - O
            // DOWN |   | RIGHT
            //      O   O
            //
            TileUndirectedEdge::Horizontal { left, right } => match dir {
                Direction::Up => TileUndirectedEdge::new(&self.0, right, Direction::Up),
                Direction::Down => TileUndirectedEdge::new(&self.0, left, Direction::Down),
                Direction::Right => TileUndirectedEdge::new(&self.0, right, Direction::Down),
                Direction::Left => TileUndirectedEdge::new(&self.0, left, Direction::Up),
            },
            // A vertical edge's neighbor will always be horizontal.
            //   L   U
            // O - O - O
            //     |
            // O - O - O
            //   D   R
            //
            TileUndirectedEdge::Vertical { down, up } => match dir {
                Direction::Up => TileUndirectedEdge::new(&self.0, up, Direction::Right),
                Direction::Down => TileUndirectedEdge::new(&self.0, down, Direction::Left),
                Direction::Right => TileUndirectedEdge::new(&self.0, down, Direction::Right),
                Direction::Left => TileUndirectedEdge::new(&self.0, up, Direction::Left),
            },
        }
    }
}

////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingEdgeError<TileType>(TileEdge<TileType>);

pub type GoResult<TileType> = Result<TileType, MissingEdgeError<TileType>>;

pub trait PathHelper<TileType>
where
    Self: HasSquareTiling<TileType>,
    TileType: Tile,
{
    /// Gets the tile at the end of the path.
    ///
    /// # Errors
    /// When a step of the path is invalid. Returns the edge attempted.
    fn skip_path<PathIterator>(&self, from: &TileType, path: PathIterator) -> GoResult<TileType>
    where
        PathIterator: Iterator<Item = Direction>;

    /// Returns an iterator following the path, outputting every tile along the way (including the first).
    fn follow_path<PathIterator>(
        &self,
        from: &TileType,
        path: PathIterator,
    ) -> impl Iterator<Item = GoResult<TileType>>
    where
        PathIterator: Iterator<Item = Direction>;
}

impl<Space, TileType> PathHelper<TileType> for Space
where
    Space: HasSquareTiling<TileType>,
    TileType: Tile,
{
    fn skip_path<PathIterator>(&self, from: &TileType, mut path: PathIterator) -> GoResult<TileType>
    where
        PathIterator: Iterator<Item = Direction>,
    {
        path.try_fold(*from, |current, dir| {
            self.go(&current, dir).ok_or({
                MissingEdgeError(TileEdge {
                    tile: current,
                    edge: dir,
                })
            })
        })
    }

    fn follow_path<PathIterator>(
        &self,
        from: &TileType,
        path: PathIterator,
    ) -> impl Iterator<Item = GoResult<TileType>>
    where
        PathIterator: Iterator<Item = Direction>,
    {
        let tail = path.scan(Ok(*from), |status, dir| {
            if let Ok(current) = status {
                *status = self.go(current, dir).ok_or(MissingEdgeError(TileEdge {
                    tile: *current,
                    edge: dir,
                }));
            }
            Some(*status)
        });

        [Ok(*from)].into_iter().chain(tail)
    }
}
