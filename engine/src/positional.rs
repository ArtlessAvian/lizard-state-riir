use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl RelativePosition {
    pub fn new(dx: i32, dy: i32) -> Self {
        Self { dx, dy }
    }

    pub fn length(self) -> u32 {
        u32::max(self.dx.unsigned_abs(), self.dy.unsigned_abs())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
