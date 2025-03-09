use engine::actions::known_serializable::KnownUnaimedAction;
use godot::prelude::*;

/// Very, very sussy.
/// We don't know the type of Gd<Resource> but we assume its something that has method "wrap".
/// That method returns this as Variant, which is opaque to Godot, but readable by us since we know the type.
#[derive(GodotClass, Debug)]
#[class(no_init)]
pub(super) struct GodotWrappedAction {
    pub action: KnownUnaimedAction,
}

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActionSet {
    base: Base<Resource>,
    #[export]
    actions: Array<Option<Gd<Resource>>>, // hopefully all impl MoveTrait
}

impl ActionSet {
    #[must_use]
    pub fn to_vec(&self) -> Vec<KnownUnaimedAction> {
        self.actions
            .iter_shared()
            .flatten()
            .map(|mut x| {
                x.call("wrap", &[])
                    .to::<Gd<GodotWrappedAction>>()
                    .bind()
                    .action
                    .clone()
            })
            .collect()
    }
}

#[godot_api]
impl IResource for ActionSet {
    // Only returns false, since we're using this to verify and not do properties.
    fn set_property(&mut self, property: StringName, value: Variant) -> bool {
        if property != c"actions".into() {
            // we're just looking to verify actions.
            return false;
        }
        if let Ok(cast) = value.try_to::<Array<Option<Gd<Resource>>>>() {
            if let Some(i) = cast
                .iter_shared()
                .position(|x| x.is_some_and(|y| !y.has_method("wrap")))
            {
                godot_error!(
                    "action[{}] does not have method wrap, in ActionSet {}",
                    i,
                    self.base().get_path()
                );
                return false;
            }

            if let Some(i) = cast.iter_shared().position(|x| {
                x.is_some_and(|y| {
                    y.clone()
                        .call("wrap", &[])
                        .try_to::<Gd<GodotWrappedAction>>()
                        .is_err()
                })
            }) {
                godot_error!(
                    "action[{}] does not have method wrap, in ActionSet {}",
                    i,
                    self.base().get_path()
                );
                return false;
            }

            if let Some(i) = cast.iter_shared().position(|x| x.is_none()) {
                godot_warn!(
                    "action[{}] is null, in ActionSet {}",
                    i,
                    self.base().get_path()
                );
                return false;
            }
        }
        false
    }
}
