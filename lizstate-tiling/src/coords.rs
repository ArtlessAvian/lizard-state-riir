use crate::direction::Direction;

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

    #[must_use]
    pub fn step(self, dir: Direction) -> Option<Self> {
        let mut out = self;
        match dir {
            Direction::Up => {
                out.y = out.y.checked_add(1)?;
            }
            Direction::Down => {
                out.y = out.y.checked_add(-1)?;
            }
            Direction::Right => {
                out.x = out.x.checked_add(1)?;
            }
            Direction::Left => {
                out.x = out.x.checked_add(-1)?;
            }
        }
        Some(out)
    }
}
