use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut, Drop};

use crate::cell::MyCell;

#[derive(Copy, Clone)]
enum RefState {
    Unshared,
    Shared(usize),
    Exclusive,
}

pub struct MyRefCell<T> {
    value: UnsafeCell<T>,
    state: MyCell<RefState>,
}

// This is implied by UnsafeCell:
// impl<T> !Sync for MyRefCell<T> {};

impl<T> MyRefCell<T> {
    pub fn new(value: T) -> MyRefCell<T> {
        MyRefCell {
            value: UnsafeCell::new(value),
            state: MyCell::new(RefState::Unshared)
        }
    }

    pub fn replace(&self, value: T) -> T {
        if let Some(ref mut borrow_mut) = self.borrow_mut() {
            std::mem::replace(&mut *borrow_mut, value)
        } else {
            panic!()
        }
    }

    pub fn borrow(&self) -> Option<MyRef<'_, T>> {
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Shared(1));
                Some(MyRef::new(&self))
            },
            RefState::Shared(n) => {
                self.state.set(RefState::Shared(n + 1));
                Some(MyRef::new(&self))
            },
            RefState::Exclusive => None,
        }
    }

    pub fn borrow_mut(&self) -> Option<MyRefMut<'_, T>> {
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Exclusive);
                Some(MyRefMut::new(&self))
            },
            _ => None,
        }
    }
}

pub struct MyRef<'refcell, T> {
    refcell: &'refcell MyRefCell<T>,
}

impl<T> MyRef<'_, T> {
    pub fn new(refcell: &MyRefCell<T>) -> MyRef<T> {
        MyRef { refcell }
    }
}

impl<T> Deref for MyRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> Drop for MyRef<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Unshared | RefState::Exclusive => unreachable!(),
            RefState::Shared(1) => {
                self.refcell.state.set(RefState::Unshared);
            },
            RefState::Shared(n) => {
                self.refcell.state.set(RefState::Shared(n - 1));
            },
        }
    }
}

pub struct MyRefMut<'refcell, T> {
    refcell: &'refcell MyRefCell<T>,
}

impl<T> MyRefMut<'_, T> {
    pub fn new(refcell: &MyRefCell<T>) -> MyRefMut<T> {
        MyRefMut { refcell }
    }
}

impl<T> Deref for MyRefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> DerefMut for MyRefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.refcell.value.get() }
    }
}

impl<T> Drop for MyRefMut<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Unshared | RefState::Shared(_) => unreachable!(),
            RefState::Exclusive => {
                self.refcell.state.set(RefState::Unshared);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let cell = MyRefCell::new(1);

        // Assert borrow works
        {
            let borrow1 = cell.borrow().unwrap();
            assert_eq!(*borrow1, 1);

            let borrow2 = cell.borrow().unwrap();
            assert_eq!(*borrow2, 1);

            assert_eq!(*borrow1, *borrow2);
        }

        // Assert borrow mut works
        {
            let mut borrow_mut = cell.borrow_mut().unwrap();
            *borrow_mut = 2;
        }
        {
            let borrow = cell.borrow().unwrap();
            assert_eq!(*borrow, 2);
        }

        // Assert replace works
        cell.replace(3);
        {
            let borrow = cell.borrow().unwrap();
            assert_eq!(*borrow, 3);
        }
    }
}
