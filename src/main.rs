mod actions;
mod data;

use std::rc::Rc;

use crate::actions::AttackRightAction;
use crate::actions::EveryoneGoRightAction;
use crate::actions::GoRightAction;
use crate::data::ActionTrait;
use crate::data::CommandTrait;
use crate::data::Entity;
use crate::data::Floor;

fn main() {
    let mut floor = Floor::new();
    [Entity { x: 5 }, Entity { x: 4 }]
        .map(Rc::new)
        .iter()
        .for_each(|e| floor = floor.add_entity(e.clone()));

    {
        let player_ref = floor.get_player();
        let go_right = GoRightAction {};
        let go_right_command = go_right.verify_action(&floor, &player_ref).unwrap();
        floor = go_right_command.do_action(&floor);
        assert_eq!(floor.get_player().x, 6);
    }
    {
        let other_ref = floor.get_someone();
        let go_right = GoRightAction {};
        floor = go_right
            .verify_action(&floor, &other_ref)
            .unwrap()
            .do_action(&floor);
    }
    {
        let other_ref = floor.get_someone();
        let go_right = GoRightAction {};
        assert!(go_right.verify_action(&floor, &other_ref).is_none());
    }
    {
        let other_ref = floor.get_someone();
        let attack_right_command = AttackRightAction {}
            .verify_action(&floor, &other_ref)
            .unwrap();
        floor = attack_right_command.do_action(&floor);
    }
    {
        let player_ref = floor.get_player();
        let all_go_right = EveryoneGoRightAction;
        floor = all_go_right
            .verify_action(&floor, &player_ref)
            .unwrap()
            .do_action(&floor);
        assert_eq!(floor.get_player().x, 7);
    }
}
