use std::ops::Deref;

use crate::core::{HasPoints, Mobject, Shape};

use super::{easing::Easing, Animatable, HasAnimation};

pub struct Animation<T, U, V>
where
    U: HasPoints,
    V: HasPoints,
    T: HasAnimation,
{
    // ref to animatable
    mob: Mobject<U>,
    initial: Shape<U>,
    target: Shape<V>,
    ref_initial: Mobject<U>,
    ref_target: Mobject<V>,
    duration: f32,
    easing: Box<dyn Easing>,
    anim: T,
}

impl<T, U, V> Animatable for Animation<T, U, V>
where
    U: HasPoints,
    V: HasPoints,
    T: HasAnimation,
{
    fn apply(&self, time: f32) -> bool {
        if time > self.duration {
            return false;
        }
        let progress = (time / self.duration).clamp(0.0, 1.0);
        self.mob
            .borrow_mut()
            .interpolate(&self.initial, &self.target, self.easing.ease(progress));
        true
    }

    fn begin(&mut self) {
        // self.anim.begin();
    }
}
