mod actions;
mod data;

use std::rc::Rc;

use crate::{
    actions::{EveryoneGoRightAction, GoRightAction},
    data::{ActionTrait, CommandTrait, Entity, Floor},
};

fn main() {
    let mut floor = Floor {
        entities: [Entity { x: 5 }, Entity { x: 5 }].map(Rc::new).to_vec(),
    };

    let player_ref = floor.get_player();
    let go_right = GoRightAction {};
    let go_right_command = go_right.verify_action(&floor, player_ref).unwrap();
    assert_eq!(go_right_command.do_action(&mut floor), Ok(()));
    assert_eq!(floor.get_player().x, 6);

    let other_ref = floor.get_someone();
    assert!(go_right.verify_action(&floor, other_ref).is_none());

    // let player_ref = floor.get_player();
    // let all_go_right = EveryoneGoRightAction;
    // assert_eq!(all_go_right.do_action(&mut floor, player_ref), Ok(()));
    // assert_eq!(floor.get_player().x, 7);
}
