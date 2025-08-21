use std::fmt::Debug;
use std::rc::Rc;

pub struct SerializableAction<TraitObject: ?Sized>(pub Rc<TraitObject>);

impl<T: ?Sized> Debug for SerializableAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SerializableExternal")
            .finish_non_exhaustive()
    }
}

impl<T: ?Sized> Clone for SerializableAction<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
