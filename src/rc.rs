use std::ops::{Deref, Drop};
use std::ptr::NonNull;

use crate::cell::MyCell;

struct MyRcInner<T> {
    value: T,
    refcount: MyCell<usize>,
}

impl<T> MyRcInner<T> {
    pub fn new(value: T) -> MyRcInner<T> {
        MyRcInner {
            value,
            refcount: MyCell::new(1),
        }
    }
}

pub struct MyRc<T> {
    inner: NonNull<MyRcInner<T>>,
}

impl<T> MyRc<T> {
    pub fn new(value: T) -> MyRc<T> {
        let inner = Box::new(MyRcInner::new(value));
        MyRc {
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
        }
    }
}

impl<T> Deref for MyRc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let inner = unsafe { self.inner.as_ref() };
        &inner.value
    }
}

impl<T> Clone for MyRc<T> {
    fn clone(&self) -> MyRc<T> {
        let inner = unsafe { self.inner.as_ref() };
        let refcount = inner.refcount.get();
        inner.refcount.set(refcount + 1);

        MyRc {
            inner: self.inner,
        }
    }
}

impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_ref() };
        let refcount = inner.refcount.get();
        if refcount == 1 {
            eprintln!("Dropping last reference...");
            drop(inner);
            let _ = unsafe { Box::from_raw(self.inner.as_ptr()) };
        } else {
            inner.refcount.set(refcount - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Foo {
        bar: usize,
    }

    impl Drop for Foo {
        fn drop(&mut self) {
            eprintln!("Foo is being dropped!");
        }
    }

    #[test]
    fn it_works() {
        let foo = Foo { bar: 1 };

        let rc = MyRc::new(foo);
        assert_eq!(rc.bar, 1);

        let rc_inner = unsafe { rc.inner.as_ref() };
        assert_eq!(rc_inner.refcount.get(), 1);

        let rc1 = rc.clone();
        assert_eq!(rc_inner.refcount.get(), 2);
        assert_eq!(rc1.bar, rc.bar);

        let rc1_inner = unsafe { rc1.inner.as_ref() };
        assert_eq!(rc1_inner.refcount.get(), 2);

        drop(rc);
        assert_eq!(rc1_inner.refcount.get(), 1);
        assert_eq!(rc1.bar, 1);

        eprintln!("Foo should NOT have been dropped yet!");
        drop(rc1);
        eprintln!("Foo should have been dropped by now!");
    }
}
