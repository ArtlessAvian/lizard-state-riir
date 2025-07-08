#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct CartesianCoords {
    pub x: i32,
    pub y: i32,
}

impl CartesianCoords {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
