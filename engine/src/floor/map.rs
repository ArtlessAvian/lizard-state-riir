pub mod vision;

use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use std::collections::HashMap;
use std::rc::Rc;

use crate::positional::fov::StrictFOV;
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

    // TODO: Move responsibility to new struct.
    pub fn get_vision(&self, pos: &AbsolutePosition) -> HashMap<AbsolutePosition, FloorTile> {
        // HACK: StrictFOV doesn't make sense for vision. You can *infer* extra data (what is/isn't a wall) from what is returned.
        // HACK: Avoid expensive construction on every call!
        let fov: StrictFOV = StrictFOV::new(5);
        let mut tiles = fov.get_field_of_view_tiles(*pos, 5, |x| !self.is_tile_floor(&x));
        // honestly this probably makes this slower for small radius
        tiles.sort_by_key(|x| (x.x, x.y));
        tiles.dedup();

        let mut out: HashMap<AbsolutePosition, FloorTile> = HashMap::new();
        for tile in tiles {
            out.insert(tile, self.get_tile(&tile).clone());
        }
        out
    }
}

impl Default for FloorMap {
    fn default() -> Self {
        Self::new()
    }
}
