use engine::actions::known_serializable::KnownUnaimedAction;
use godot::prelude::*;

use super::known_unaimed::ToKnownUnaimed;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActionSet {
    base: Base<Resource>,
    #[export]
    actions: Array<DynGd<Resource, dyn ToKnownUnaimed>>, // hopefully all impl MoveTrait
}

impl ActionSet {
    #[must_use]
    pub fn to_vec(&self) -> Vec<KnownUnaimedAction> {
        self.actions
            .iter_shared()
            .map(|x| x.dyn_bind().to_known_action())
            .collect()
    }
}
