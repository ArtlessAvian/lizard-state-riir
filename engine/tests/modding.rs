#![allow(private_interfaces)]

use std::borrow::Cow;
/// Presuming I either split up crates.
/// Or if someone actually wants to mod. That'd be crazy.
use std::rc::Rc;

use engine::actions::ActionError;
use engine::actions::ActionTrait;
use engine::actions::BoxedCommand;
use engine::actions::CommandTrait;
use engine::actions::DeserializeActionTrait;
use engine::actions::SerializeActionTrait;
use engine::actions::events::ExitEvent;
use engine::actions::events::FloorEvent;
use engine::actions::known_serializable::KnownAction;
use engine::actions::serializable_wrapper::SerializableAction;
use engine::entity::Entity;
use engine::entity::EntityId;
use engine::floor::Floor;
use engine::floor::FloorUpdate;
use engine::strategy::NullStrategy;
use rkyv::Archive;
use rkyv::Archived;
use rkyv::Deserialize;
use rkyv::Infallible;
use rkyv::Serialize;
use rkyv::ser::Serializer;
use rkyv::ser::serializers::AllocSerializer;
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;

#[derive(PartialEq, Eq, Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(TypeName))]
struct TestAction {}

#[archive_dyn(deserialize)]
impl ActionTrait for TestAction {
    fn verify_and_box<'a>(
        &self,
        floor: &Cow<'a, Floor>,
        subject_id: EntityId,
    ) -> Result<BoxedCommand<'a>, ActionError> {
        Ok(BoxedCommand::new_from_trait(TestCommand {
            parsed_floor: floor.clone(),
            subject_id,
        }))
    }
}

impl ActionTrait for Archived<TestAction> {
    fn verify_and_box<'a>(
        &self,
        floor: &Cow<'a, Floor>,
        subject_id: EntityId,
    ) -> Result<BoxedCommand<'a>, ActionError> {
        Deserialize::<TestAction, _>::deserialize(self, &mut Infallible)
            .unwrap()
            .verify_and_box(floor, subject_id)
    }
}

struct TestCommand<'a> {
    parsed_floor: Cow<'a, Floor>,
    subject_id: EntityId,
}

impl CommandTrait for TestCommand<'_> {
    fn do_action(self) -> FloorUpdate {
        FloorUpdate::new(self.parsed_floor.as_ref().clone()).log_each(vec![
            FloorEvent::Exit(
                ExitEvent {
                    subject: self.subject_id,
                }
            );
            3
        ])
    }
}

fn expect_test_action_side_effects(type_erased: Rc<dyn ActionTrait>) {
    let floor = Floor::new_minimal();
    let (update, id) = floor
        .add_entity(Entity {
            state: engine::entity::EntityState::Ok { next_round: 0 },
            pos: engine::positional::AbsolutePosition::new(0, 0),
            health: 10,
            max_energy: 10,
            energy: 10,
            moveset: Vec::new(),
            strategy: NullStrategy {}.into(),
            is_player_controlled: true,
            is_player_friendly: true,
            payload: "Hello!".to_owned(),
        })
        .split_pair();
    let update = update.bind(|f| {
        type_erased
            .verify_and_box(&Cow::Borrowed(&f), id)
            .unwrap()
            .do_action()
    });
    let dingus = update.into_both().1;
    assert_eq!(dingus, vec![FloorEvent::Exit(ExitEvent { subject: id }); 3])
}

#[test]
fn test_test_action() {
    expect_test_action_side_effects(Rc::new(TestAction {}));
}

#[test]
fn rkyv_roundtrip() {
    let action = TestAction {};
    let known_external = KnownAction::External(SerializableAction(Rc::new(action.clone())));
    {
        let mut serializer = AllocSerializer::<256>::default();
        serializer.serialize_value(&known_external).unwrap();

        let bytes = serializer.into_serializer().into_inner();
        let archived = unsafe { rkyv::archived_root::<KnownAction>(&bytes[..]) };
        // TODO: Validate bytes somehow.

        let deserialized: KnownAction = archived
            .deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new())
            .unwrap();

        expect_test_action_side_effects(Rc::new(deserialized));
    }
}
