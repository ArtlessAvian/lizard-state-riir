pub mod vision;

use rkyv::with::Skip;
use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::pathfinding::PathfindingContext;
use crate::positional::AbsolutePosition;

// TODO: Decide whether to use non_exhaustive.
#[derive(Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[non_exhaustive]
pub enum FloorTile {
    FLOOR,
    WALL,
}

impl FloorTile {
    #[must_use]
    pub fn is_tile_floor(&self) -> bool {
        // clean (and obvious) but more floors will be added ig.
        matches!(self, FloorTile::FLOOR)
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct FloorMap {
    pub tiles: Rc<HashMap<AbsolutePosition, FloorTile>>,
    // TODO: Link tiles' lifetime to pathfinder's lifetime.
    // TODO: Figure out rkyv interaction with pathfinder.
    #[with(Skip)]
    pub pathfinder: Option<Rc<RefCell<PathfindingContext>>>,
    pub default: FloorTile,
}

impl FloorMap {
    #[must_use]
    pub fn new_empty() -> Self {
        FloorMap {
            tiles: Rc::new(HashMap::new()),
            default: FloorTile::FLOOR, // "outdoors" map.
            pathfinder: Some(Rc::new(RefCell::new(PathfindingContext::new(
                Box::new(|_| true),
                Box::new(AbsolutePosition::distance),
            )))),
        }
    }

    #[must_use]
    pub fn new_with_tiles(tiles: HashMap<AbsolutePosition, FloorTile>) -> Self {
        let tiles = Rc::new(tiles);
        FloorMap {
            tiles: Rc::clone(&tiles),
            default: FloorTile::WALL, // "indoors" map.
            pathfinder: Some(Rc::new(RefCell::new(PathfindingContext::new(
                Box::new(move |pos| !tiles.get(&pos).unwrap_or(&FloorTile::WALL).is_tile_floor()),
                Box::new(AbsolutePosition::distance),
            )))),
        }
    }

    #[must_use]
    pub fn get_tile(&self, pos: &AbsolutePosition) -> &FloorTile {
        self.tiles.get(pos).unwrap_or(&self.default)
    }

    #[must_use]
    pub fn is_tile_floor(&self, pos: &AbsolutePosition) -> bool {
        // clean (and obvious) but more floors will be added ig.
        self.get_tile(pos).is_tile_floor()
    }

    #[must_use]
    pub fn get_step(
        &self,
        start: AbsolutePosition,
        destination: AbsolutePosition,
    ) -> Option<AbsolutePosition> {
        if let Some(pathfinder) = &self.pathfinder {
            if pathfinder.borrow_mut().find_path(start, destination) {
                return pathfinder.borrow().get_step(start, destination);
            }
        }
        None
    }
}

impl Default for FloorMap {
    fn default() -> Self {
        Self::new_empty()
    }
}
