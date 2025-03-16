use super::Branch;
use super::CaveSystem;
use super::Room;

#[derive(Clone, Copy)]
#[must_use]
pub enum CaveSystemNode<'a> {
    Reservoir {
        system: &'a CaveSystem,
    },
    InRoom {
        system: &'a CaveSystem,
        branch: &'a Branch,
        room: &'a Room,
    },
}

impl CaveSystemNode<'_> {
    #[must_use]
    pub fn get_neighbors(&self) -> Vec<Self> {
        let mut out = Vec::new();

        match self {
            CaveSystemNode::Reservoir { system } => {
                out.push(CaveSystemNode::InRoom {
                    system,
                    branch: &system.main,
                    room: &system.main.rooms[system.main.get_river_end()],
                });
                for other in system.others.iter().flatten() {
                    out.push(CaveSystemNode::InRoom {
                        system,
                        branch: other,
                        room: &other.rooms[other.get_river_end()],
                    });
                }
            }
            CaveSystemNode::InRoom {
                system,
                branch,
                room,
            } => {
                match room.flow_in {
                    super::FlowIn::Source => {}
                    super::FlowIn::One(a) => out.push(CaveSystemNode::InRoom {
                        system,
                        branch,
                        room: &branch.rooms[a],
                    }),
                    super::FlowIn::Two(a, b) => {
                        out.push(CaveSystemNode::InRoom {
                            system,
                            branch,
                            room: &branch.rooms[a],
                        });
                        out.push(CaveSystemNode::InRoom {
                            system,
                            branch,
                            room: &branch.rooms[b],
                        });
                    }
                }

                match room.flow_out {
                    super::FlowOut::None => {
                        if std::ptr::eq(&branch.rooms[branch.get_river_end()], *room) {
                            out.push(CaveSystemNode::Reservoir { system });
                        }
                    }
                    super::FlowOut::One(a) => out.push(CaveSystemNode::InRoom {
                        system,
                        branch,
                        room: &branch.rooms[a],
                    }),
                    super::FlowOut::Two(a, b) => {
                        out.push(CaveSystemNode::InRoom {
                            system,
                            branch,
                            room: &branch.rooms[a],
                        });
                        out.push(CaveSystemNode::InRoom {
                            system,
                            branch,
                            room: &branch.rooms[b],
                        });
                    }
                }
            }
        }

        out
    }
}

impl PartialEq for CaveSystemNode<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Reservoir { system: l_system }, Self::Reservoir { system: r_system }) => {
                std::ptr::eq(l_system, r_system)
            }
            (
                Self::InRoom {
                    system: l_system,
                    branch: l_branch,
                    room: l_room,
                },
                Self::InRoom {
                    system: r_system,
                    branch: r_branch,
                    room: r_room,
                },
            ) => {
                std::ptr::eq(l_system, r_system)
                    && std::ptr::eq(l_branch, r_branch)
                    && std::ptr::eq(l_room, r_room)
            }
            _ => false,
        }
    }
}
impl Eq for CaveSystemNode<'_> {}
