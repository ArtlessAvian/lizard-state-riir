use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::hash::Hash;

use crate::positional::AbsolutePosition;
use crate::positional::RelativePosition;

struct SymmetricMatrix<K, V>(HashMap<(K, K), V>);

impl<K: Hash + Ord + Copy, V: Copy> SymmetricMatrix<K, V> {
    fn normalize_key(key: (K, K)) -> (K, K) {
        if key.0 <= key.1 {
            key
        } else {
            (key.1, key.0)
        }
    }

    fn contains_key(&self, key: (K, K)) -> bool {
        self.0.contains_key(&Self::normalize_key(key))
    }

    fn iter(&self) -> impl Iterator<Item = ((K, K), V)> + '_ {
        self.0.iter().map(|(k, v)| (*k, *v))
    }

    fn iter_symmetric_pairs(&self) -> impl Iterator<Item = ((K, K), V)> + '_ {
        self.iter().chain(
            self.iter()
                .filter(|(k, _)| k.0 != k.1)
                .map(|(k, v)| ((k.1, k.0), v)),
        )
    }

    fn iter_row(&self, half_key: K) -> impl Iterator<Item = ((K, K), V)> + '_ {
        self.iter().filter(move |(k, _)| k.0 == half_key).chain(
            self.iter()
                .filter(move |(k, _)| k.1 == half_key)
                .map(|(k, v)| ((k.1, k.0), v)),
        )
    }

    fn insert(&mut self, k: (K, K), v: V) -> Option<V> {
        self.0.insert(Self::normalize_key(k), v)
    }

    fn get(&self, k: (K, K)) -> Option<&V> {
        self.0.get(&Self::normalize_key(k))
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
    previous: Option<AbsolutePosition>,
    known_cost_so_far: u32,
    estimated_cost: u32,
}

impl Ord for PartialPath {
    fn cmp(&self, other: &Self) -> Ordering {
        let cost_comparison = self.estimated_cost.cmp(&other.estimated_cost).reverse();
        if cost_comparison != Ordering::Equal {
            return cost_comparison;
        }
        // In an estimate tie, we want to try the path is further along the path.
        self.known_cost_so_far.cmp(&other.known_cost_so_far)
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
    }
}

struct PathfindingContext {
    // Config stuff. Don't mess with this after construction.
    blocked: Box<dyn FnMut(AbsolutePosition) -> bool>,
    heuristic: Box<dyn FnMut(AbsolutePosition, AbsolutePosition) -> u32>,
    // Partial information that gets filled out over calls.
    // Imagine the Floyd-Warshall algorithm's matrix.
    known_distance: SymmetricMatrix<AbsolutePosition, u32>,
    // Some position on the path between the key.
    // If you're wondering how to get there, ask for a position on the path between here and there.
    // Don't read directly.
    step_between: SymmetricMatrix<AbsolutePosition, AbsolutePosition>,
}

impl PathfindingContext {
    fn find_path(&mut self, start: AbsolutePosition, destination: AbsolutePosition) -> bool {
        if self.known_distance.contains_key((start, destination)) {
            return true;
        }

        let mut frontier = BinaryHeap::<PartialPath>::new();
        // Resume a previous search.
        for (path, cost) in self.known_distance.iter_row(start) {
            frontier.push(PartialPath {
                tile: path.1,
                previous: None, // do not overwrite step since we already know this.
                known_cost_so_far: cost,
                estimated_cost: cost + (self.heuristic)(path.1, destination),
            });
        }

        if frontier.is_empty() {
            // Begin a new search.
            frontier.push(PartialPath {
                tile: start,
                previous: None,
                known_cost_so_far: 0,
                estimated_cost: (self.heuristic)(start, destination),
            });
        }

        // Traverse and discover new facts.
        while let Some(partial_path) = frontier.pop() {
            // If the item was popped, we know we have the shortest path. This may rewrite info from a resumed search.
            self.known_distance
                .insert((start, partial_path.tile), partial_path.known_cost_so_far);
            // Avoid overwriting data from resuming. The previous is guaranteed to be between start and destination.
            partial_path.previous.inspect(|intermediate| {
                if !self.step_between.contains_key((start, partial_path.tile)) {
                    self.step_between
                        .insert((start, partial_path.tile), *intermediate);
                }
            });

            // Equivalently since we just inserted,
            // `self.known_distance.contains_key(&(start, destination))`
            if partial_path.tile == destination {
                return true;
            }

            // Give up. Magic number tbh.
            if partial_path.estimated_cost > 5 + (self.heuristic)(start, destination) {
                return false;
            }

            // Add known paths as extensions.
            for (path, cost) in self.known_distance.iter_row(partial_path.tile) {
                if !self.known_distance.contains_key((start, path.1)) {
                    frontier.push(PartialPath {
                        tile: path.1,
                        previous: Some(path.0),
                        known_cost_so_far: partial_path.known_cost_so_far + cost,
                        estimated_cost: partial_path.known_cost_so_far
                            + cost
                            + (self.heuristic)(path.1, destination),
                    });
                }
            }

            // Add direct neighbors.
            for delta in RelativePosition::list_all_length(1) {
                let neighbor = partial_path.tile + delta;
                if self.known_distance.contains_key((start, neighbor)) {
                    continue;
                }
                if (self.blocked)(neighbor) {
                    continue;
                }

                frontier.push(PartialPath {
                    tile: neighbor,
                    previous: Some(partial_path.tile),
                    known_cost_so_far: partial_path.known_cost_so_far + 1,
                    estimated_cost: partial_path.known_cost_so_far
                        + 1
                        + (self.heuristic)(neighbor, destination),
                });
            }
        }
        false
    }

    fn get_step(
        &self,
        start: AbsolutePosition,
        destination: AbsolutePosition,
    ) -> Option<AbsolutePosition> {
        dbg!((start, destination));
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
}

#[test]
fn permissive_diagonal() {
    let mut context = PathfindingContext {
        blocked: Box::new(|_| false),
        heuristic: Box::new(AbsolutePosition::distance),
        known_distance: SymmetricMatrix::default(),
        step_between: SymmetricMatrix::default(),
    };

    let start = AbsolutePosition::new(0, 0);
    let destination = AbsolutePosition::new(5, 5);
    context.find_path(start, destination);

    assert_eq!(context.known_distance.get((start, destination)), Some(&5));
    assert_eq!(
        context.get_step(start, destination),
        Some(AbsolutePosition::new(1, 1))
    );
}

#[test]
fn permissive_bad_heuristic() {
    let mut context = PathfindingContext {
        blocked: Box::new(|_| false),
        // consistently underesimates true distance.
        // devolves into dijkstra's
        heuristic: Box::new(|_, _| 0),
        known_distance: SymmetricMatrix::default(),
        step_between: SymmetricMatrix::default(),
    };

    let start = AbsolutePosition::new(0, 0);
    let destination = AbsolutePosition::new(5, 5);
    context.find_path(start, destination);

    assert_eq!(context.known_distance.get((start, destination)), Some(&5));
    assert_eq!(
        context.get_step(start, destination),
        Some(AbsolutePosition::new(1, 1))
    );
}

#[test]
fn no_path() {
    let mut context = PathfindingContext {
        blocked: Box::new(|_| true),
        heuristic: Box::new(AbsolutePosition::distance),
        known_distance: SymmetricMatrix::default(),
        step_between: SymmetricMatrix::default(),
    };

    assert!(!context.find_path(AbsolutePosition::new(0, 0), AbsolutePosition::new(5, 0)));
}

#[test]
fn no_path_infinite_frontier() {
    let mut context = PathfindingContext {
        blocked: Box::new(|pos| pos.x == 1 && pos.y == 1),
        heuristic: Box::new(AbsolutePosition::distance),
        known_distance: SymmetricMatrix::default(),
        step_between: SymmetricMatrix::default(),
    };

    assert!(!context.find_path(AbsolutePosition::new(0, 0), AbsolutePosition::new(1, 1)));
}

#[test]
fn resume_run() {
    let mut context = PathfindingContext {
        blocked: Box::new(|_| false),
        heuristic: Box::new(AbsolutePosition::distance),
        known_distance: SymmetricMatrix::default(),
        step_between: SymmetricMatrix::default(),
    };

    let start = AbsolutePosition::new(0, 0);
    {
        let destination = AbsolutePosition::new(3, 3);
        context.find_path(start, destination);

        assert_eq!(context.known_distance.get((start, destination)), Some(&3));
        assert_eq!(
            context.get_step(start, destination),
            Some(AbsolutePosition::new(1, 1))
        );
    }

    {
        let destination = AbsolutePosition::new(5, 5);
        context.find_path(start, destination);

        assert_eq!(context.known_distance.get((start, destination)), Some(&5));
        assert_eq!(
            context.get_step(start, destination),
            Some(AbsolutePosition::new(1, 1))
        );
    }
}

#[test]
fn resume_run_backwards() {
    let mut context = PathfindingContext {
        blocked: Box::new(|_| false),
        heuristic: Box::new(AbsolutePosition::distance),
        known_distance: SymmetricMatrix::default(),
        step_between: SymmetricMatrix::default(),
    };

    {
        let start = AbsolutePosition::new(0, 0);
        let destination = AbsolutePosition::new(3, 3);
        context.find_path(start, destination);

        assert_eq!(context.known_distance.get((start, destination)), Some(&3));
        assert_eq!(
            context.get_step(start, destination),
            Some(AbsolutePosition::new(1, 1))
        );
    }

    {
        let start = AbsolutePosition::new(5, 5);
        let destination = AbsolutePosition::new(-2, -2);
        context.find_path(start, destination);

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
            .map(move |(x, _)| AbsolutePosition::new(x.try_into().unwrap(), y.try_into().unwrap()))
    })
    .collect::<std::collections::HashSet<AbsolutePosition>>();

    let mut context = PathfindingContext {
        blocked: Box::new(move |pos| maze.contains(&pos)),
        heuristic: Box::new(AbsolutePosition::distance),
        known_distance: SymmetricMatrix::default(),
        step_between: SymmetricMatrix::default(),
    };

    let start = AbsolutePosition::new(0, 0);
    let destination = AbsolutePosition::new(6, 6);
    context.find_path(start, destination);

    assert_eq!(context.known_distance.get((start, destination)), Some(&11));
    assert_eq!(
        context.get_step(start, destination),
        Some(AbsolutePosition::new(1, 1))
    );
}
