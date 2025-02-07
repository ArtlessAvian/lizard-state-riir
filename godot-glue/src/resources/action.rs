use engine::actions::characters::axolotl_nano::EnterSmiteStanceAction;
use engine::actions::characters::max_tegu::ForwardHeavyAction;
use engine::actions::characters::max_tegu::TrackingAction;
use engine::actions::example::DoubleHitAction;
use engine::actions::example::EnterStanceAction;
use godot::prelude::*;

use super::actionset::GodotWrappedAction;

macro_rules! expose_action_to_godot {
    ( $type:ident, $cons:expr ) => {
        #[derive(GodotClass, Debug)]
        #[class(init, base=Resource)]
        pub(crate) struct $type {
            base: Base<Resource>,
        }

        #[godot_api]
        impl $type {
            #[func]
            fn wrap() -> Gd<GodotWrappedAction> {
                Gd::from_object(GodotWrappedAction {
                    action: $cons.into(),
                })
            }
        }
    };
}

expose_action_to_godot!(DoubleHit, DoubleHitAction);
expose_action_to_godot!(EnterStance, EnterStanceAction);
expose_action_to_godot!(ForwardHeavy, ForwardHeavyAction);
expose_action_to_godot!(Tracking, TrackingAction);

expose_action_to_godot!(EnterSmiteStance, EnterSmiteStanceAction);
