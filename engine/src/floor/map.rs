pub mod vision;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use std::collections::HashMap;
use std::rc::Rc;

use crate::positional::AbsolutePosition;

// TODO: Decide whether to use non_exhaustive.
#[derive(Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[non_exhaustive]
pub enum FloorTile {
    FLOOR,
    WALL,
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct FloorMap {
    pub tiles: Rc<HashMap<AbsolutePosition, FloorTile>>,
    pub default: FloorTile,
}

impl FloorMap {
    pub fn new() -> Self {
        FloorMap {
            tiles: Rc::new(HashMap::new()),
            default: FloorTile::FLOOR, // "outdoors" map.
        }
    }

    pub fn get_tile(&self, pos: &AbsolutePosition) -> &FloorTile {
        self.tiles.get(pos).unwrap_or(&self.default)
    }

    pub fn is_tile_floor(&self, pos: &AbsolutePosition) -> bool {
        // clean (and obvious) but more floors will be added ig.
        matches!(self.get_tile(pos), FloorTile::FLOOR)
    }
}

impl Default for FloorMap {
    fn default() -> Self {
        Self::new()
    }
}
