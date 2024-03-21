use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn length(self) -> u32 {
        u32::max(self.dx.unsigned_abs(), self.dy.unsigned_abs())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbsolutePosition {
    pub x: i32,
    pub y: i32,
}

impl AbsolutePosition {
    pub fn distance(self, other: AbsolutePosition) -> u32 {
        (self - other).length()
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
