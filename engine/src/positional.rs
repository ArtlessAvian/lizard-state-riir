pub mod algorithms;
pub mod fov;

use std::ops::{Add, Mul, Sub};

use rkyv::{Archive, Deserialize, Serialize};

/// An offset.
/// A Vector2i, like AbsolutePosition.
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Archive,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
)]
#[archive_attr(derive(Hash, PartialEq, Eq, Debug))]
pub struct RelativePosition {
    pub dx: i32,
    pub dy: i32,
}

impl Add for RelativePosition {
    type Output = RelativePosition;

    fn add(self, rhs: Self) -> Self::Output {
        RelativePosition {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}

impl Mul<RelativePosition> for i32 {
    type Output = RelativePosition;

    fn mul(self, rhs: RelativePosition) -> Self::Output {
        RelativePosition {
            dx: rhs.dx * self,
            dy: rhs.dy * self,
        }
    }
}

impl RelativePosition {
    pub fn new(dx: i32, dy: i32) -> Self {
        Self { dx, dy }
    }

    pub fn length(self) -> u32 {
        u32::max(self.dx.unsigned_abs(), self.dy.unsigned_abs())
    }
}

/// A Vector2i.
///
/// The distance function is the L-infinity norm, aka Chebyshev distance.
/// This was chosen because it behaves nicely at distance 1 (compared to L-1 norm)
/// The alternatives are:
/// * The L-1 norm has direct diagonals at 2 distance. You'd need to attack 1-2 range, like a Fire Emblem mage.
/// * The L-2 norm has non-integer values. Diagonals are >1 distance. You can round but it stops being a metric space. It produces nice circles!
/// * Tabletop rules: Every second diagonal costs 2. Again stops being a metric space. Nice integer values! (and octagons!)

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Archive, Serialize, Deserialize, PartialOrd, Ord,
)]
#[archive_attr(derive(Hash, PartialEq, Eq, Debug))]
pub struct AbsolutePosition {
    pub x: i32,
    pub y: i32,
}

impl AbsolutePosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: AbsolutePosition) -> u32 {
        (self - other).length()
    }
}

// Only right addition is defined intentionally.
impl Add<RelativePosition> for AbsolutePosition {
    type Output = AbsolutePosition;

    fn add(self, rhs: RelativePosition) -> Self::Output {
        AbsolutePosition {
            x: self.x + rhs.dx,
            y: self.y + rhs.dy,
        }
    }
}

impl Sub for AbsolutePosition {
    type Output = RelativePosition;

    fn sub(self, rhs: Self) -> Self::Output {
        RelativePosition {
            dx: self.x - rhs.x,
            dy: self.y - rhs.y,
        }
    }
}

/// An alternate representation of a RelativePosition that hides direction.
/// Not very useful outside of algorithms.
/// Don't make public.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
struct OctantRelative {
    run: u32,
    rise: u32,  // rise < run
    octant: u8, // Treat as a black box.
}

impl OctantRelative {
    fn ignore_octant(rise: u32, run: u32) -> Self {
        Self {
            run,
            rise,
            octant: 0,
        }
    }

    // If you are working in an octant, you are not likely to exit the octant.
    fn in_same_octant(&self, rise: u32, run: u32) -> Self {
        Self {
            rise,
            run,
            octant: self.octant,
        }
    }

    // TODO:
    // fn make_all_octants(&self) -> [OctantRelative; 8] {
    //     (0..8).map(|octant| OctantRelative { octant, ..*self })
    // }
}

impl From<RelativePosition> for OctantRelative {
    fn from(value: RelativePosition) -> Self {
        // let mut octant = 0;
        // if value.dy < 0 {
        //     octant += 4;
        //     value.dy *= -1;
        // }
        // if value.dx < 0 {
        //     octant += 2;
        //     value.dx *= -1;
        // }
        // if value.dx < value.dy {
        //     octant += 1;
        //     (value.dx, value.dy) = (value.dy, value.dx)
        // }
        Self {
            run: u32::max(value.dx.unsigned_abs(), value.dy.unsigned_abs()),
            rise: u32::min(value.dx.unsigned_abs(), value.dy.unsigned_abs()),
            octant: if value.dy < 0 { 0b100 } else { 0 }
                + if value.dx < 0 { 0b010 } else { 0 }
                + if value.dy.abs() > value.dx.abs() {
                    0b001
                } else {
                    0
                },
        }
    }
}

impl From<OctantRelative> for RelativePosition {
    fn from(value: OctantRelative) -> Self {
        let (mut dx, mut dy) = (value.run as i32, value.rise as i32);

        if value.octant & 0b001 != 0 {
            (dx, dy) = (dy, dx);
        }
        dx *= if value.octant & 0b010 != 0 { -1 } else { 1 };
        dy *= if value.octant & 0b100 != 0 { -1 } else { 1 };

        Self { dx, dy }
    }
}

#[cfg(test)]
#[test]
fn test_octant_from_into() {
    for dx in -10..11 {
        for dy in -10..11 {
            let relative = RelativePosition { dx, dy };
            assert_eq!(
                relative,
                RelativePosition::from(OctantRelative::from(relative))
            );
        }
    }
}
