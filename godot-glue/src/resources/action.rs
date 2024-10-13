use engine::actions::characters::max_tegu::ForwardHeavyAction;
use engine::actions::characters::max_tegu::TrackingAction;
use engine::actions::example::DoubleHitAction;
use engine::actions::example::EnterStanceAction;
use godot::prelude::*;

use super::actionset::GodotWrappedAction;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct DoubleHit {
    base: Base<Resource>,
}

#[godot_api]
impl DoubleHit {
    #[func]
    fn wrap() -> Gd<GodotWrappedAction> {
        Gd::from_object(GodotWrappedAction {
            action: DoubleHitAction.into(),
        })
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct EnterStance {
    base: Base<Resource>,
}

#[godot_api]
impl EnterStance {
    #[func]
    fn wrap() -> Gd<GodotWrappedAction> {
        Gd::from_object(GodotWrappedAction {
            action: EnterStanceAction.into(),
        })
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ForwardHeavy {
    base: Base<Resource>,
}

#[godot_api]
impl ForwardHeavy {
    #[func]
    fn wrap() -> Gd<GodotWrappedAction> {
        Gd::from_object(GodotWrappedAction {
            action: ForwardHeavyAction.into(),
        })
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct Tracking {
    base: Base<Resource>,
}

#[godot_api]
impl Tracking {
    #[func]
    fn wrap() -> Gd<GodotWrappedAction> {
        Gd::from_object(GodotWrappedAction {
            action: TrackingAction.into(),
        })
    }
}
