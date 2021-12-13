use std::cell::UnsafeCell;

pub struct MyCell<T> {
    value: UnsafeCell<T>,
}

// This is implied by UnsafeCell:
// impl<T> !Sync for MyCell<T> {};

impl<T> MyCell<T> {
    pub fn new(value: T) -> MyCell<T> {
        MyCell { value: UnsafeCell::new(value) }
    }

    pub fn set(&self, value: T) {
        unsafe { *self.value.get() = value }
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { *self.value.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let cell = MyCell::new(1);
        assert_eq!(cell.get(), 1);

        cell.set(2);
        assert_eq!(cell.get(), 2);
    }
}
