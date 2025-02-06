pub mod vision;

use std::cell::OnceCell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rkyv::with::Skip;
use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;

use crate::pathfinding::PathfindingContext;
use crate::positional::AbsolutePosition;

// TODO: Decide whether to use non_exhaustive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
#[non_exhaustive]
pub enum FloorTile {
    Floor,
    Wall,
}

impl FloorTile {
    #[must_use]
    pub(crate) fn is_tile_floor(self) -> bool {
        // clean (and obvious) but more floors will be added ig.
        matches!(self, FloorTile::Floor)
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(Debug))]
pub struct FloorMap {
    pub tiles: Rc<HashMap<AbsolutePosition, FloorTile>>,
    // TODO: Link tiles' lifetime to pathfinder's lifetime.
    // TODO: Figure out rkyv interaction with pathfinder.
    #[with(Skip)]
    pub pathfinder: Rc<OnceCell<RefCell<PathfindingContext<'static>>>>,
    pub default: FloorTile,
}

impl FloorMap {
    #[must_use]
    pub(crate) fn new_empty() -> Self {
        FloorMap {
            tiles: Rc::new(HashMap::new()),
            default: FloorTile::Floor, // "outdoors" map.
            pathfinder: Rc::new(OnceCell::new()),
        }
    }

    #[must_use]
    pub fn new_with_tiles(tiles: HashMap<AbsolutePosition, FloorTile>) -> Self {
        FloorMap {
            tiles: Rc::new(tiles),
            default: FloorTile::Wall, // "indoors" map.
            pathfinder: Rc::new(OnceCell::new()),
        }
    }

    #[must_use]
    pub(crate) fn get_tile(&self, pos: AbsolutePosition) -> &FloorTile {
        self.tiles.get(&pos).unwrap_or(&self.default)
    }

    #[must_use]
    pub(crate) fn is_tile_floor(&self, pos: AbsolutePosition) -> bool {
        self.get_tile(pos).is_tile_floor()
    }

    #[must_use]
    pub(crate) fn get_step(
        &self,
        start: AbsolutePosition,
        destination: AbsolutePosition,
    ) -> Option<AbsolutePosition> {
        let lazy = self.pathfinder.get_or_init(|| {
            let tiles = Rc::clone(&self.tiles);
            let default = self.default;
            RefCell::new(PathfindingContext::new(
                Box::new(move |pos| !tiles.get(&pos).unwrap_or(&default).is_tile_floor()),
                Box::new(AbsolutePosition::distance),
            ))
        });

        if lazy.borrow_mut().find_path(start, destination) {
            return lazy.borrow().get_step(start, destination);
        }
        None
    }
}

impl Default for FloorMap {
    fn default() -> Self {
        Self::new_empty()
    }
}
