use engine::actions::SerializableUnaimedAction;
use godot::prelude::*;

/// Very, very sussy.
/// We don't know the type of Gd<Resource> but we assume its something that has method "wrap".
/// That method returns this as Variant, which is opaque to Godot, but readable by us since we know the type.
#[derive(GodotClass, Debug)]
#[class(no_init)]
pub(super) struct GodotWrappedAction {
    pub action: SerializableUnaimedAction,
}

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActionSet {
    base: Base<Resource>,
    #[export]
    actions: Array<Option<Gd<Resource>>>, // hopefully all impl MoveTrait
}

impl ActionSet {
    pub fn to_vec(&self) -> Vec<SerializableUnaimedAction> {
        self.actions
            .iter_shared()
            .flatten()
            .map(|mut x| {
                x.call("wrap".into(), &[])
                    .to::<Gd<GodotWrappedAction>>()
                    .bind()
                    .action
                    .clone()
            })
            .collect()
    }
}
