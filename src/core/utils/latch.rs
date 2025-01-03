use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct Latch<T> {
    value: T,
    latch: bool,
}

impl<T> Deref for Latch<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Latch<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.latch = true;
        &mut self.value
    }
}

impl<T> Latch<T> {
    pub fn new_reset(value: T) -> Self {
        Self {
            value,
            latch: false,
        }
    }
    pub fn new_set(value: T) -> Self {
        Self { value, latch: true }
    }

    pub fn reset(&mut self) -> bool {
        if self.latch {
            self.latch = false;
            return true;
        }
        false
    }
}

use std::cell::RefCell;
use std::rc::Rc;

trait MyTrait {
    fn do_something(&self);
}

struct MyStruct<T> {
    value: T,
}

impl<T> MyTrait for MyStruct<T> {
    fn do_something(&self) {
        println!("Value:");
    }
}

fn main() {
    let mut vec: Vec<Rc<RefCell<dyn MyTrait>>> = Vec::new();

    let item1 = Rc::new(RefCell::new(MyStruct { value: 42 }));
    let item2 = Rc::new(RefCell::new(MyStruct { value: 10.0 }));

    vec.push(item1);
    vec.push(item2);

    for item in &vec {
        item.borrow().do_something();
    }
}
