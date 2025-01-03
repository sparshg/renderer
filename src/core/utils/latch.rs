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
