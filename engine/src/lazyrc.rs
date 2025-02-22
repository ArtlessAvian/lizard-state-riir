use std::borrow::Borrow;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub enum LazyRc<'a, T> {
    Rc(Rc<T>),
    Owned(T),
    Borrowed(&'a T),
}

impl<T: Clone> LazyRc<'_, T> {
    pub fn unwrap_or_clone(self) -> T {
        match self {
            LazyRc::Rc(x) => Rc::unwrap_or_clone(x),
            LazyRc::Owned(x) => x,
            LazyRc::Borrowed(x) => x.clone(),
        }
    }

    pub fn into_static_self(self) -> LazyRc<'static, T> {
        match self {
            LazyRc::Rc(x) => LazyRc::Rc(x),
            LazyRc::Owned(x) => LazyRc::Owned(x),
            LazyRc::Borrowed(x) => LazyRc::Owned(x.clone()),
        }
    }

    pub fn clone_to_static_self(&self) -> LazyRc<'static, T> {
        match self {
            LazyRc::Rc(x) => LazyRc::Rc(Rc::clone(x)),
            LazyRc::Owned(x) => LazyRc::Owned(x.clone()),
            LazyRc::Borrowed(x) => LazyRc::Owned((*x).clone()),
        }
    }

    pub fn clone_to_rc(&self) -> Rc<T> {
        match self {
            LazyRc::Rc(x) => Rc::clone(x),
            LazyRc::Owned(x) => Rc::new(x.clone()),
            LazyRc::Borrowed(x) => Rc::new((*x).clone()),
        }
    }
}

impl<T> Borrow<T> for LazyRc<'_, T> {
    fn borrow(&self) -> &T {
        match self {
            LazyRc::Rc(x) => x.borrow(),
            LazyRc::Owned(x) => x.borrow(),
            LazyRc::Borrowed(x) => x,
        }
    }
}

impl<T> Deref for LazyRc<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            LazyRc::Rc(x) => x,
            LazyRc::Owned(x) => x,
            LazyRc::Borrowed(x) => x,
        }
    }
}

impl<T> AsRef<T> for LazyRc<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            LazyRc::Rc(x) => x,
            LazyRc::Owned(x) => x,
            LazyRc::Borrowed(x) => x,
        }
    }
}
