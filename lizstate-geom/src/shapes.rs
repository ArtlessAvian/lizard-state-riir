use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::Range;

use crate::positional::InsideOctant;
use crate::positional::RelativeOctantified;
use crate::positional::RelativePosition;

trait Shape {
    fn list_relative(&self) -> HashSet<RelativePosition>;
    fn contains_relative(&self, x: RelativePosition) -> bool;
    fn union_all(&self) -> impl Shape;
}

/// Tiles within a max radius.
/// Excludes the zero vector.
struct Ball<const R: u8>();

impl<const R: u8> Shape for Ball<R> {
    fn list_relative(&self) -> HashSet<RelativePosition> {
        let len = R.into();
        (1..=len)
            .flat_map(|run| {
                (0..8).flat_map(move |octant| {
                    (0..=len).map(move |rise| {
                        RelativeOctantified {
                            inside: InsideOctant { run, rise },
                            octant,
                        }
                        .into()
                    })
                })
            })
            .collect::<HashSet<RelativePosition>>()
    }

    fn contains_relative(&self, x: RelativePosition) -> bool {
        x.length() <= R.into()
    }

    fn union_all(&self) -> impl Shape {
        Ball::<R>()
    }
}

/// Subset of a Ball, at least N tiles closest in angle to a ray.
/// Will max out at 180 degrees!
struct Sector<const R: u8> {
    ray: RelativePosition,
    top_n: usize,
}

impl<const R: u8> Shape for Sector<R> {
    fn list_relative(&self) -> HashSet<RelativePosition> {
        let circle = Ball::<R>().list_relative();

        let mut positive_dot = circle
            .into_iter()
            .filter(|&x| dot(x, self.ray) >= 0)
            .collect::<Vec<_>>();

        positive_dot.sort_by(|&left, &right| angle_comparator(left, right, self.ray));

        let yeag = positive_dot
            .iter()
            .map(|&x| (x, dot(x, self.ray)))
            .collect::<Vec<_>>();
        dbg!(yeag);

        let nth_maybe = positive_dot.get(self.top_n - 1);
        let cutoff = nth_maybe.map_or(usize::MAX, |&nth| {
            positive_dot.partition_point(|&key| match angle_comparator(key, nth, self.ray) {
                Ordering::Equal | Ordering::Less => true,
                Ordering::Greater => false,
            })
        });

        positive_dot.into_iter().take(cutoff).collect()
    }

    fn contains_relative(&self, x: RelativePosition) -> bool {
        self.list_relative().contains(&x)
    }

    fn union_all(&self) -> impl Shape {
        Ball::<R>()
    }
}

/// Tiles within a radius range.
struct Donut(Range<u8>);

/// Top N tiles of a donut closest in angle to a ray.
struct Macaroni(Range<u8>);

fn dot(a: RelativePosition, b: RelativePosition) -> i32 {
    a.dx * b.dx + a.dy * b.dy
}

/// Sorts from closest to least.
fn angle_comparator(
    left: RelativePosition,
    right: RelativePosition,
    target: RelativePosition,
) -> Ordering {
    (dot(left, target).pow(2) * dot(right, right))
        .cmp(&(dot(right, target).pow(2) * dot(left, left)))
        .reverse()
}

#[cfg(test)]
mod test {
    use super::Sector;
    use super::Shape;
    use crate::positional::RelativePosition;
    use crate::shapes::Ball;
    use crate::shapes::dot;

    #[test]
    fn dot_is_dot() {
        assert_eq!(
            dot(RelativePosition::new(0, 0), RelativePosition::new(123, 456)),
            0
        );
        assert_eq!(
            dot(RelativePosition::new(1, 0), RelativePosition::new(123, 456)),
            123
        );
        assert_eq!(
            dot(RelativePosition::new(0, 1), RelativePosition::new(123, 456)),
            456
        );
        assert_eq!(
            dot(RelativePosition::new(1, 1), RelativePosition::new(123, 456)),
            579
        );
    }

    #[test]
    fn circle_is_circle() {
        let dingus = Ball::<1>().list_relative();

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                assert!(
                    dingus.contains(&RelativePosition::new(dx, dy)),
                    "{dx}, {dy}"
                );
            }
        }
    }

    #[test]
    fn sector_is_sector() {
        let dingus = Sector::<1> {
            ray: RelativePosition::new(1, 0),
            top_n: 3,
        }
        .list_relative();

        dbg!(&dingus);

        assert_eq!(dingus.len(), 3);
        assert!(dingus.contains(&RelativePosition::new(1, -1)));
        assert!(dingus.contains(&RelativePosition::new(1, 0)));
        assert!(dingus.contains(&RelativePosition::new(1, 1)));
    }

    #[test]
    fn sector_includes_when_equal() {
        let dingus = Sector::<1> {
            ray: RelativePosition::new(1, 0),
            top_n: 2,
        }
        .list_relative();

        // Even though we asked for 2, the 2nd closest is equally close as the 3rd.
        // The 4th is not included of course.
        assert_eq!(dingus.len(), 3);
        assert!(dingus.contains(&RelativePosition::new(1, -1)));
        assert!(dingus.contains(&RelativePosition::new(1, 0)));
        assert!(dingus.contains(&RelativePosition::new(1, 1)));
    }
}
