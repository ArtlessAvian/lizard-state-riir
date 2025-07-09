use core::ops::Deref;
use std::rc::Rc;

use crate::custom_space::CustomSpace;

// Newtype around `Rc<CustomSpace>`.
// Derefs to `CustomSpace`, so you can still access the `IsTilingGraph` trait.
#[derive(Clone)]
pub struct SharedCustomSpace(Rc<CustomSpace>);

impl Deref for SharedCustomSpace {
    type Target = CustomSpace;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
