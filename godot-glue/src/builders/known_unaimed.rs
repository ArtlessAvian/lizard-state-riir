use engine::actions::characters::axolotl_nano::EnterSmiteStanceAction;
use engine::actions::characters::max_tegu::ForwardHeavyAction;
use engine::actions::characters::max_tegu::TrackingAction;
use engine::actions::example::DoubleHitAction;
use engine::actions::example::EnterStanceAction;
use engine::actions::known_serializable::KnownUnaimedAction;
use godot::prelude::*;

pub trait ToKnownUnaimed {
    fn to_known_action(&self) -> KnownUnaimedAction;
}

macro_rules! expose_action_to_godot {
    ( $type:ident, $cons:expr ) => {
        #[derive(GodotClass, Debug)]
        #[class(init, base=Resource)]
        pub struct $type {
            base: Base<Resource>,
        }

        #[godot_dyn]
        impl ToKnownUnaimed for $type {
            fn to_known_action(&self) -> KnownUnaimedAction {
                $cons.into()
            }
        }
    };
}

expose_action_to_godot!(DoubleHit, DoubleHitAction);
expose_action_to_godot!(EnterStance, EnterStanceAction);
expose_action_to_godot!(ForwardHeavy, ForwardHeavyAction);
expose_action_to_godot!(Tracking, TrackingAction);

expose_action_to_godot!(EnterSmiteStance, EnterSmiteStanceAction);
