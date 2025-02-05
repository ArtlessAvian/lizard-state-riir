use engine::entity::Entity;
use engine::entity::EntityState;
use engine::floor::Floor;
use engine::positional::AbsolutePosition;
use engine::strategy::NullStrategy;
use rkyv::rancor;

#[test]
fn serialize_deserialize() {
    let floor = Floor::new_with_all_systems();
    let floor = floor
        .add_entity(Entity {
            state: EntityState::Ok {
                next_round: 0x2aaa_aaaa,
            },
            pos: AbsolutePosition::new(0x3bbb_bbbb, 0x4ccc_cccc),
            health: 0x5d,
            max_energy: 0x6e,
            energy: 0x7f,
            moveset: Vec::new(),
            strategy: NullStrategy.into(),
            is_player_controlled: false,
            is_player_friendly: true,
            payload: "Hello there!".into(),
        })
        .split_pair()
        .0
        .into_both()
        .0;

    let bytes = rkyv::api::high::to_bytes::<rancor::Error>(&floor).unwrap();
    let _deserialized = rkyv::api::high::from_bytes::<Floor, rancor::Error>(&bytes).unwrap();

    // // We can't easily check the equality of deserialized and floor.
    // // Oh well.

    // // Fun stuff.
    // assert_eq!(bytes.len(), 9476);
}
