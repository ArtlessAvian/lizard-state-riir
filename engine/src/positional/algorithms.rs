// TODO: "Continuous" rays/segments?
// TODO: Field of view calculation.
// TODO: Consider the difference between seeing a tile, seeing something on the tile, and hitting something on the tile.
// (Maybe only the last is symmetric? Maybe symmetry is not valuable?)
// (Currently the only gameplay value is revealing the map in a neat way.)
// (There's no cover mechanics for example.)

use super::InsideOctant;
use super::RelativeOctantified;
use super::RelativePosition;

/// A symmetric segment.
///
/// The symmetry is a bit pointless, but it looks good.
/// If you at A draw a line to C which passes through B without hitting a wall,
/// Sometimes B cannot draw a line passing through A without hitting a wall.
///
/// ```markdown
/// Fig A      Fig B-1     Fig B-2
/// ##########  ##!#######  !!########
/// ..A---....  ..A---....  ..A!!.....
/// ######B--C  ######B...  #####!B...
/// ```
///
/// TODON'T: Make an infinte iterator for Rays, maybe.
/// They aren't symmetric, but that's not a problem.
/// There isn't much of a use to an infinite ray that can't be solved with a long segment?
/// If one were to be made anyways, maybe there could be a bias "up" and bias "down,"
/// similar to the bias in the implementation for segments.
pub struct Segment {}

impl Segment {
    pub(super) fn calculate(
        target: InsideOctant,
    ) -> (Vec<InsideOctant>, Option<Vec<InsideOctant>>) {
        let mut tiles = Vec::new();

        let mut rise = 0;
        for run in 0..=target.run {
            // Reimplemented from myself at https://github.com/ArtlessAvian/lizard-state/blob/development/Engine/GridHelper.cs#L81-L87
            // Based on Tran Thong "A symmetric linear algorithm for line segment generation."

            // We want to move up when y+1 is more accurate than y.
            // if x' * (y + 0.5) < y' * x
            // You can imagine the + 0.5 being an error bar. If its less than the error margin, than y+1 must be within 0.5.
            // (We know the slope is <= 1, so the error can't grow faster)
            // We can make both sides unsigned int only!
            // if self.target.run * (2 * rise + 1) < self.target.rise * 2 * run {
            //     rise += 1;
            // }

            // If the error is exactly 0.5, then both y and y+1 are tolerable.
            // There are annoying cases where there's a lot of these. So we bias all of the points by rotating around the middle.
            // (Again the goal is to produce symmetric lines.)
            // If we decrease the slope, we overestimate the start. If we increase the slope, we overestimate the end. Oh well.
            // Then there are at least two lines. The middle can still be 0.5, so theres one line for y and y+1.
            // Remember that the formula for a line through a point is y - py = m (x - px). We get:
            // (y + 0.5 - y'/2) < (y'/x' + epsilon) * (x - x'/2)
            // Again we can simplify to ints, with extreme scaling for the epsilon, but its annoyingly signed.

            // Instead we can just do this simpler equivalent thing lol.
            // Thanks Rust for motivating this with annoying u32 conversions with `as i32`.
            if 2 * run <= target.run {
                if target.run * (2 * rise + 1) < target.rise * 2 * run {
                    rise += 1;
                }
            } else if target.run * (2 * rise + 1) <= target.rise * 2 * run {
                rise += 1;
            }

            tiles.push(InsideOctant::new(rise, run));
        }

        let mut alt = None;
        if target.rise % 2 != 0 && target.run % 2 == 0 {
            let mut clone = tiles.clone();
            let mid = clone.len() / 2; // int division intentional.
            clone[mid].rise += 1;
            alt = Some(clone);
        }

        (tiles, alt)
    }

    pub(crate) fn calculate_relative(
        target: RelativePosition,
    ) -> (Vec<RelativePosition>, Option<Vec<RelativePosition>>) {
        let target_octantified: RelativeOctantified = target.into();
        let target_in_octant: InsideOctant = target_octantified.inside;

        let (a, b) = Self::calculate(target_in_octant);
        let a_relative = a
            .into_iter()
            .map(|x| target_octantified.in_same_octant(x))
            .map(RelativePosition::from)
            .collect();
        let b_relative = b.map(|x| {
            x.into_iter()
                .map(|x| target_octantified.in_same_octant(x))
                .map(RelativePosition::from)
                .collect()
        });
        (a_relative, b_relative)
    }
}

#[cfg(test)]
mod test {
    use crate::positional::algorithms::Segment;
    use crate::positional::InsideOctant;
    use crate::positional::RelativePosition;

    #[test]
    fn test_cardinal() {
        let (seg_octant, none) = Segment::calculate(InsideOctant::new(0, 10));
        assert!(none.is_none());

        for (i, el) in seg_octant.iter().enumerate() {
            assert_eq!(el.run as usize, i);
            assert_eq!(el.rise, 0);
        }
    }

    #[test]
    fn test_diagonal() {
        let (seg_octant, none) = Segment::calculate(InsideOctant::new(10, 10));
        assert!(none.is_none());

        dbg!(seg_octant.clone());
        for (i, el) in seg_octant.iter().enumerate() {
            assert_eq!(el.run as usize, i);
            assert_eq!(el.rise as usize, i);
        }
    }

    #[test]
    fn test_midpoint() {
        let (seg_octant, some) = Segment::calculate(InsideOctant::new(1, 2));
        let alt = some.unwrap();

        assert_eq!(seg_octant[0], alt[0]);
        assert_ne!(seg_octant[1], alt[1]);
        assert_eq!(seg_octant[2], alt[2]);
    }

    #[test]
    fn test_symmetry() {
        let target = RelativePosition::new(23, 7);
        let (mut forwards, none) = Segment::calculate_relative(target);
        assert!(none.is_none());

        let mut backwards = forwards
            .iter()
            .map(|x| target + -1 * *x)
            .collect::<Vec<RelativePosition>>();

        forwards.sort();
        backwards.sort();
        assert_eq!(forwards, backwards);
    }

    #[test]
    fn test_symmetry_alt() {
        let target = RelativePosition::new(24, 7);

        let (mut forwards, some) = Segment::calculate_relative(target);
        let mut backwards = some
            .unwrap()
            .into_iter()
            .map(|x| target + -1 * x)
            .collect::<Vec<RelativePosition>>();

        forwards.sort();
        backwards.sort();
        assert_eq!(forwards, backwards);
    }

    #[test]
    fn test_annoying_slope() {
        // Often passes through (x, y + 0.5) for some integers x and y.
        let target = RelativePosition::new(10, 5);

        let (mut forwards, some) = Segment::calculate_relative(target);
        let mut backwards = some
            .unwrap()
            .into_iter()
            .map(|x| target + -1 * x)
            .collect::<Vec<RelativePosition>>();

        forwards.sort();
        backwards.sort();
        assert_eq!(forwards, backwards);
    }

    #[test]
    fn test_octant() {
        // Often passes through (x, y + 0.5) for some integers x and y.
        let target = RelativePosition::new(-5, -10);

        let (mut forwards, some) = Segment::calculate_relative(target);
        let mut backwards = some
            .unwrap()
            .into_iter()
            .map(|x| target + -1 * x)
            .collect::<Vec<RelativePosition>>();

        forwards.sort();
        backwards.sort();
        assert_eq!(forwards, backwards);
    }
}
