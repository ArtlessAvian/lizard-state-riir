use std::collections::BTreeMap;
use std::collections::HashMap;

use graph_representation::CaveSystem;
use graph_representation::traversal::CaveSystemNode;

use crate::positional::AbsolutePosition;

pub mod graph_representation;

struct ValidCaveSystemInPlane<'a> {
    yeah: CaveSystemInPlane<'a>,
}

struct CaveSystemInPlane<'a> {
    system: CaveSystem,
    positions: HashMap<CaveSystemNode<'a>, AbsolutePosition>,
}

impl<'a> CaveSystemInPlane<'a> {
    fn new(system: CaveSystem) -> Self {
        let mut out = Self {
            system,
            positions: HashMap::new(),
        };

        for (i, node) in out.system.get_all_nodes().into_iter().enumerate() {
            let i = i as i32;
            out.positions.insert(node, AbsolutePosition::new(i, i * i));
        }

        out
    }

    fn is_valid(&self) -> bool {
        for (a, b) in self.system.get_all_edges() {
            for (c, d) in self.system.get_all_edges() {
                if a == c || a == d || b == c || b == d {
                    continue;
                }
                if i32::max(self.positions[&a].x, self.positions[&b].x)
                    < i32::min(self.positions[&c].x, self.positions[&d].x)
                {
                    continue;
                }
            }
        }

        true
    }
}
