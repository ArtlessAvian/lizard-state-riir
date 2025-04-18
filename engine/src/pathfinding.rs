use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use tracing::instrument;

use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

#[derive(Debug)]
struct SymmetricMatrix<K, V>(HashMap<K, HashMap<K, V>>);

impl<K: Hash + Ord + Copy, V: Copy> SymmetricMatrix<K, V> {
    fn contains_key(&self, key: (K, K)) -> bool {
        self.0
            .get(&key.0)
            .is_some_and(|inner| inner.contains_key(&key.1))
    }

    // `key.0` will always be the half_key passed in.
    fn iter_row(&'_ self, half_key: K) -> impl Iterator<Item = ((K, K), V)> + '_ {
        self.0
            .get(&half_key)
            .into_iter()
            .flat_map(move |inner| inner.iter().map(move |(k, v)| ((half_key, *k), *v)))
    }

    fn insert(&mut self, k: (K, K), v: V) -> Option<V> {
        self.0.entry(k.0).or_default().insert(k.1, v);
        self.0.entry(k.1).or_default().insert(k.0, v)
    }

    fn get(&self, k: (K, K)) -> Option<&V> {
        self.0.get(&k.0).and_then(|inner| inner.get(&k.1))
    }
}

impl<K, V> Default for SymmetricMatrix<K, V> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

#[derive(Eq, Debug, Clone)]
struct PartialPath {
    tile: AbsolutePosition,
    // Only populated if this is not resumed from a previous run.
    previous: AbsolutePosition,
    known_cost_so_far: u32,
    estimated_cost: u32,
    // Stabilize paths.
    known_diagonal: u32,
    estimated_diagonal: u32,
}

impl Ord for PartialPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.estimated_cost
            .cmp(&other.estimated_cost)
            .reverse()
            .then_with(|| {
                self.estimated_diagonal
                    .cmp(&other.estimated_diagonal)
                    .reverse()
            })
            .then_with(||
            // In an estimate tie, we want to try the path is further along the path.
            self.known_cost_so_far.cmp(&other.known_cost_so_far))
    }
}

impl PartialOrd for PartialPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PartialPath {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost == other.estimated_cost
            && self.estimated_diagonal == other.estimated_diagonal
            && self.known_cost_so_far == other.known_cost_so_far
    }
}

pub struct NoPath;

pub struct PathfindingContext<'a> {
    // Config stuff. Don't mess with this after construction.
    blocked: Rc<dyn Fn(AbsolutePosition) -> bool + 'a>,
    // Always an underestimate. Therefore, if NoPath is returned, then there really is no path.
    heuristic: Box<dyn FnMut(AbsolutePosition, AbsolutePosition) -> Result<u32, NoPath> + 'a>,
    // Partial information that gets filled out over calls.
    // Imagine the Floyd-Warshall algorithm's matrix.
    known_distance: SymmetricMatrix<AbsolutePosition, u32>,
    diagonal_steps: SymmetricMatrix<AbsolutePosition, u32>,
    // Some position on the path between the key.
    // If you're wondering how to get there, ask for a position on the path between here and there.
    // Don't read directly.
    step_between: SymmetricMatrix<AbsolutePosition, AbsolutePosition>,
}

impl<'a> PathfindingContext<'a> {
    #[must_use]
    pub fn new(
        blocked: impl Fn(AbsolutePosition) -> bool + 'a,
        mut heuristic: impl FnMut(AbsolutePosition, AbsolutePosition) -> u32 + 'a,
    ) -> Self {
        Self::new_disconnected(blocked, move |a, b| Ok(heuristic(a, b)))
    }

    #[must_use]
    pub fn new_disconnected(
        blocked: impl Fn(AbsolutePosition) -> bool + 'a,
        heuristic: impl FnMut(AbsolutePosition, AbsolutePosition) -> Result<u32, NoPath> + 'a,
    ) -> Self {
        Self {
            blocked: Rc::new(blocked),
            heuristic: Box::new(heuristic),
            known_distance: SymmetricMatrix::default(),
            diagonal_steps: SymmetricMatrix::default(),
            step_between: SymmetricMatrix::default(),
        }
    }

    pub fn create_subgraph<'into>(
        &'into mut self,
        also_blocked: impl Fn(AbsolutePosition) -> bool + 'into,
    ) -> PathfindingContext<'into> {
        let my_blocked = Rc::clone(&self.blocked);

        let blocked = move |x| my_blocked(x) || also_blocked(x);
        let heuristic = |x, y| {
            let _ = self.find_path(x, y);
            self.get_distance(x, y)
        };

        PathfindingContext::<'into>::new_disconnected(blocked, heuristic)
    }
}

impl PathfindingContext<'_> {
    #[must_use]
    #[instrument(skip_all)]
    pub fn find_path(&mut self, start: AbsolutePosition, destination: AbsolutePosition) -> bool {
        if self.known_distance.contains_key((start, destination)) {
            return true;
        }

        let Ok(optimistic_estimate) = (self.heuristic)(start, destination) else {
            return false;
        };

        // Keep the frontier small.
        let within_limit =
            |partial: &PartialPath| partial.estimated_cost <= 5 + optimistic_estimate;

        // Note to programmer: don't be clever and try to reuse self.known_distance.
        let mut visited = HashSet::<AbsolutePosition>::new();

        let mut frontier = BinaryHeap::<PartialPath>::new();
        frontier.push(PartialPath {
            tile: start,
            previous: start,
            known_cost_so_far: 0,
            known_diagonal: 0,
            estimated_cost: optimistic_estimate,
            estimated_diagonal: 0,
        });

        while let Some(partial_path) = frontier.pop() {
            if visited.contains(&partial_path.tile) {
                continue;
            }
            visited.insert(partial_path.tile);

            // If the item was popped, we know we have the shortest path.
            self.known_distance
                .insert((start, partial_path.tile), partial_path.known_cost_so_far);
            self.diagonal_steps
                .insert((start, partial_path.tile), partial_path.known_diagonal);
            // The previous is guaranteed to be between start and destination.
            if !self.step_between.contains_key((start, partial_path.tile)) {
                self.step_between
                    .insert((start, partial_path.tile), partial_path.previous);
            }

            // Equivalently since we just inserted,
            // `self.known_distance.contains_key(&(start, destination))`
            if partial_path.tile == destination {
                return true;
            }

            if !within_limit(&partial_path) {
                // Ideally this should never run, since everything in the frontier is within_limit.
                // The frontier should run out of elements instead.
                // If this does run, everything else in the frontier is known to be worse.
                return false;
            }

            // Direct neighbors have a known cost.
            // This also lets us path backwards later.
            for neighbor in RelativePosition::list_all_length(1)
                .into_iter()
                .map(|delta| partial_path.tile + delta)
                .filter(|x| !(self.blocked)(*x))
            {
                let delta = neighbor - partial_path.tile;
                self.known_distance.insert((partial_path.tile, neighbor), 1);
                self.diagonal_steps.insert(
                    (partial_path.tile, neighbor),
                    u32::from(delta.dx != 0 && delta.dy != 0),
                );
                self.step_between
                    .insert((partial_path.tile, neighbor), partial_path.tile);
            }

            // Add known paths.
            let bingus = self
                .known_distance
                .iter_row(partial_path.tile)
                .filter(|(path, _)| !visited.contains(&path.1))
                .filter_map(|(path, cost)| {
                    Some(PartialPath {
                        tile: path.1,
                        previous: path.0, // path.0 is betweeen start and path.1.
                        known_cost_so_far: partial_path.known_cost_so_far + cost,
                        known_diagonal: partial_path.known_diagonal
                            + self.diagonal_steps.get(path).unwrap(),
                        estimated_cost: partial_path.known_cost_so_far
                            + cost
                            + (self.heuristic)(path.1, destination).ok()?,
                        estimated_diagonal: partial_path.known_diagonal
                            + self.diagonal_steps.get(path).unwrap()
                            + (destination - path.1).dx.unsigned_abs()
                            + (destination - path.1).dy.unsigned_abs(),
                    })
                });

            frontier.extend(bingus.filter(within_limit));
        }
        false
    }

    #[must_use]
    pub fn get_step(
        &self,
        start: AbsolutePosition,
        destination: AbsolutePosition,
    ) -> Option<AbsolutePosition> {
        if start == destination {
            Some(destination)
        } else if let Some(&intermediate) = self.step_between.get((start, destination)) {
            if intermediate == start || intermediate == destination {
                Some(destination)
            } else {
                self.get_step(start, intermediate)
            }
        } else {
            None
        }
    }

    // TODO: Probably make this part of find_path.
    pub fn get_distance(
        &self,
        start: AbsolutePosition,
        destination: AbsolutePosition,
    ) -> Result<u32, NoPath> {
        self.known_distance
            .get((start, destination))
            .copied()
            .ok_or(NoPath)
    }
}

impl Debug for PathfindingContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathfindingContext")
            .field("known_distance", &self.known_distance)
            .field("step_between", &self.step_between)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod test {
    use crate::pathfinding::NoPath;
    use crate::pathfinding::PathfindingContext;
    use crate::positional::AbsolutePosition;

    #[test]
    fn permissive_diagonal() {
        let mut context = PathfindingContext::new(|_| false, AbsolutePosition::distance);

        let start = AbsolutePosition::new(0, 0);
        let destination = AbsolutePosition::new(5, 5);
        assert!(context.find_path(start, destination));

        assert_eq!(context.known_distance.get((start, destination)), Some(&5));
        assert_eq!(
            context.get_step(start, destination),
            Some(AbsolutePosition::new(1, 1))
        );
    }

    #[test]
    fn permissive_minimize_diagonals() {
        let mut context = PathfindingContext::new(|_| false, AbsolutePosition::distance);

        let start = AbsolutePosition::new(0, 0);
        let destination = AbsolutePosition::new(5, 0);
        assert!(context.find_path(start, destination));

        assert_eq!(context.known_distance.get((start, destination)), Some(&5));
        assert_eq!(
            context.get_step(start, destination),
            Some(AbsolutePosition::new(1, 0))
        );
    }

    #[test]
    fn permissive_bad_heuristic() {
        let mut context = PathfindingContext::new(
            |_| false,
            // consistently underesimates true distance.
            // devolves into dijkstra's
            |_, _| 0,
        );

        let start = AbsolutePosition::new(0, 0);
        let destination = AbsolutePosition::new(3, 3);
        assert!(context.find_path(start, destination));

        assert_eq!(context.known_distance.get((start, destination)), Some(&3));
        assert_eq!(
            context.get_step(start, destination),
            Some(AbsolutePosition::new(1, 1))
        );
    }

    #[test]
    fn no_path() {
        let mut context = PathfindingContext::new(|_| true, AbsolutePosition::distance);

        assert!(!context.find_path(AbsolutePosition::new(0, 0), AbsolutePosition::new(5, 0)));
    }

    #[test]
    fn no_path_infinite_frontier() {
        let mut context =
            PathfindingContext::new(|pos| pos.x == 1 && pos.y == 1, AbsolutePosition::distance);

        assert!(!context.find_path(AbsolutePosition::new(0, 0), AbsolutePosition::new(1, 1)));
    }

    #[test]
    fn known_disconnected() {
        let mut context = PathfindingContext::new_disconnected(|_| false, |_, _| Err(NoPath));

        assert!(!context.find_path(AbsolutePosition::new(0, 0), AbsolutePosition::new(1, 0)));
    }

    #[test]
    fn resume_run() {
        let mut context = PathfindingContext::new(|_| false, AbsolutePosition::distance);

        let start = AbsolutePosition::new(0, 0);
        {
            let destination = AbsolutePosition::new(3, 3);
            assert!(context.find_path(start, destination));

            assert_eq!(context.known_distance.get((start, destination)), Some(&3));
            assert_eq!(
                context.get_step(start, destination),
                Some(AbsolutePosition::new(1, 1))
            );
        }

        {
            let destination = AbsolutePosition::new(5, 5);
            assert!(context.find_path(start, destination));

            assert_eq!(context.known_distance.get((start, destination)), Some(&5));
            assert_eq!(
                context.get_step(start, destination),
                Some(AbsolutePosition::new(1, 1))
            );
        }
    }

    #[test]
    fn resume_run_from_middle() {
        let mut context = PathfindingContext::new(|_| false, AbsolutePosition::distance);

        let destination = AbsolutePosition::new(3, 3);
        {
            let start: AbsolutePosition = AbsolutePosition::new(0, 0);
            assert!(context.find_path(start, destination));

            assert_eq!(context.known_distance.get((start, destination)), Some(&3));
            assert_eq!(
                context.get_step(start, destination),
                Some(AbsolutePosition::new(1, 1))
            );
        }

        {
            let start: AbsolutePosition = AbsolutePosition::new(1, 1);
            assert!(context.find_path(start, destination));

            assert_eq!(context.known_distance.get((start, destination)), Some(&2));
            assert_eq!(
                context.get_step(start, destination),
                Some(AbsolutePosition::new(2, 2))
            );
        }
    }

    #[test]
    fn resume_run_unrelated_destination() {
        let mut context = PathfindingContext::new(|_| false, AbsolutePosition::distance);

        let start: AbsolutePosition = AbsolutePosition::new(0, 0);
        {
            let destination = AbsolutePosition::new(3, 3);
            assert!(context.find_path(start, destination));

            assert_eq!(context.known_distance.get((start, destination)), Some(&3));
            assert_eq!(
                context.get_step(start, destination),
                Some(AbsolutePosition::new(1, 1))
            );
        }

        {
            let destination = AbsolutePosition::new(-3, -3);
            assert!(context.find_path(start, destination));

            assert_eq!(context.known_distance.get((start, destination)), Some(&3));
            assert_eq!(
                context.get_step(start, destination),
                Some(AbsolutePosition::new(-1, -1))
            );
        }
    }

    #[test]
    fn resume_run_backwards() {
        let mut context = PathfindingContext::new(|_| false, AbsolutePosition::distance);

        {
            let start = AbsolutePosition::new(0, 0);
            let destination = AbsolutePosition::new(3, 3);
            assert!(context.find_path(start, destination));

            assert_eq!(context.known_distance.get((start, destination)), Some(&3));
            assert_eq!(
                context.get_step(start, destination),
                Some(AbsolutePosition::new(1, 1))
            );
        }

        {
            let start = AbsolutePosition::new(5, 5);
            let destination = AbsolutePosition::new(-2, -2);
            assert!(context.find_path(start, destination));

            assert_eq!(context.known_distance.get((start, destination)), Some(&7));
            assert_eq!(
                context.get_step(start, destination),
                Some(AbsolutePosition::new(4, 4))
            );
        }
    }

    #[test]
    fn solve_maze() {
        // tbh this maze can be solved pretty greedily.
        let maze = "
        @#######
        #.##..##
        #.#.##.#
        #.#.##.#
        #.#....#
        #.#.##.#
        ##.###.#
        ########
    "
        .split_ascii_whitespace()
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars()
                .enumerate()
                .filter(|(_, c)| *c == '#')
                .map(move |(x, _)| {
                    AbsolutePosition::new(x.try_into().unwrap(), y.try_into().unwrap())
                })
        })
        .collect::<std::collections::HashSet<AbsolutePosition>>();

        let mut context =
            PathfindingContext::new(move |pos| maze.contains(&pos), AbsolutePosition::distance);

        let start = AbsolutePosition::new(0, 0);
        let destination = AbsolutePosition::new(6, 6);
        assert!(context.find_path(start, destination));

        assert_eq!(context.known_distance.get((start, destination)), Some(&11));
        assert_eq!(
            context.get_step(start, destination),
            Some(AbsolutePosition::new(1, 1))
        );
    }
}
