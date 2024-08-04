use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

#[derive(Default, Debug)]
pub struct RcMut<T>(Rc<RefCell<T>>);

impl<T> RcMut<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        (*self.0).borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        (*self.0).borrow_mut()
    }
}

impl<T> Clone for RcMut<T> {
    fn clone(&self) -> Self {
        RcMut(self.0.clone())
    }
}
