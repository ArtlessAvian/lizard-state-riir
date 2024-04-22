// TODO: "Continuous" rays/segments?
// TODO: Field of view calculation.
// TODO: Consider the difference between seeing a tile, seeing something on the tile, and hitting something on the tile.
// (Maybe only the last is symmetric? Maybe symmetry is not valuable?)
// (Currently the only gameplay value is revealing the map in a neat way.)
// (There's no cover mechanics for example.)

use super::{OctantRelative, RelativePosition};

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
pub struct Segment {
    target: OctantRelative,
}

impl Segment {
    pub fn calculate(&self) -> (Vec<OctantRelative>, Option<Vec<OctantRelative>>) {
        let mut tiles = Vec::new();

        let mut rise = 0;
        for run in 0..self.target.run + 1 {
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
            if 2 * run <= self.target.run {
                if self.target.run * (2 * rise + 1) < self.target.rise * 2 * run {
                    rise += 1;
                }
            } else if self.target.run * (2 * rise + 1) <= self.target.rise * 2 * run {
                rise += 1;
            }

            tiles.push(OctantRelative {
                run,
                rise,
                octant: self.target.octant,
            })
        }

        let mut alt = None;
        if self.target.rise % 2 != 0 && self.target.run % 2 == 0 {
            let mut clone = tiles.clone();
            let mid = clone.len() / 2; // int division intentional.
            clone[mid].rise += 1;
            alt = Some(clone);
        }

        (tiles, alt)
    }

    pub fn calculate_relative(&self) -> (Vec<RelativePosition>, Option<Vec<RelativePosition>>) {
        let (a, b) = self.calculate();
        let a_relative = a.into_iter().map(RelativePosition::from).collect();
        let b_relative = b.map(|x| x.into_iter().map(RelativePosition::from).collect());
        (a_relative, b_relative)
    }
}

#[cfg(test)]
#[test]
fn test_cardinal() {
    use super::RelativePosition;

    let seg = Segment {
        target: OctantRelative::from(RelativePosition::new(10, 0)),
    };
    let (seg_octant, none) = seg.calculate();
    assert!(none.is_none());

    for (i, el) in seg_octant.iter().enumerate() {
        assert_eq!(el.run, i as u32);
        assert_eq!(el.rise, 0);
    }
}

#[cfg(test)]
#[test]
fn test_diagonal() {
    use super::RelativePosition;

    let seg = Segment {
        target: OctantRelative::from(RelativePosition::new(10, 10)),
    };
    let (seg_octant, none) = seg.calculate();
    assert!(none.is_none());

    dbg!(seg_octant.clone());
    for (i, el) in seg_octant.iter().enumerate() {
        assert_eq!(el.run, i as u32);
        assert_eq!(el.rise, i as u32);
    }
}

#[cfg(test)]
#[test]
fn test_midpoint() {
    use super::RelativePosition;

    let seg = Segment {
        target: OctantRelative::from(RelativePosition::new(2, 1)),
    };
    let (seg_octant, some) = seg.calculate();
    let alt = some.unwrap();

    assert_eq!(seg_octant[0], alt[0]);
    assert_ne!(seg_octant[1], alt[1]);
    assert_eq!(seg_octant[2], alt[2]);
}

#[cfg(test)]
#[test]
fn test_symmetry() {
    use super::RelativePosition;

    let target = RelativePosition::new(23, 7);

    let seg = Segment {
        target: OctantRelative::from(target),
    };
    let (forwards, none) = seg.calculate_relative();
    assert!(none.is_none());

    let mut backwards = forwards
        .iter()
        .map(|x| target + -1 * *x)
        .collect::<Vec<RelativePosition>>();
    backwards.sort_by_key(|x| x.dx);

    assert_eq!(forwards, backwards);
}

#[cfg(test)]
#[test]
fn test_symmetry_alt() {
    use super::RelativePosition;

    let target = RelativePosition::new(24, 7);

    let seg = Segment {
        target: OctantRelative::from(target),
    };
    let (forwards, some) = seg.calculate_relative();
    let mut backwards = some
        .unwrap()
        .into_iter()
        .map(|x| target + -1 * x)
        .collect::<Vec<RelativePosition>>();
    backwards.sort_by_key(|x| x.dx);

    assert_eq!(forwards, backwards);
}

#[cfg(test)]
#[test]
fn test_annoying_slope() {
    use super::RelativePosition;

    // Often passes through (x, y + 0.5) for some integers x and y.
    let target = RelativePosition::new(10, 5);

    let seg = Segment {
        target: OctantRelative::from(target),
    };
    let (forwards, some) = seg.calculate_relative();
    let mut backwards = some
        .unwrap()
        .into_iter()
        .map(|x| target + -1 * x)
        .collect::<Vec<RelativePosition>>();
    backwards.sort_by_key(|x| x.dx);

    assert_eq!(forwards, backwards);
}

#[cfg(test)]
#[test]
fn test_octant() {
    use super::RelativePosition;

    // Often passes through (x, y + 0.5) for some integers x and y.
    let target = RelativePosition::new(-5, -10);

    let seg = Segment {
        target: OctantRelative::from(target),
    };
    let (forwards, some) = seg.calculate_relative();
    let mut backwards = some
        .unwrap()
        .into_iter()
        .map(|x| target + -1 * x)
        .collect::<Vec<RelativePosition>>();
    backwards.sort_by_key(|x| -x.dy);

    assert_eq!(forwards, backwards);
}
