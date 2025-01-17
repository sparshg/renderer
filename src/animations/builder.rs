use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::core::{HasPoints, Mobject, Shape};

use super::Animatable;

pub struct AnimationBuilder<T: HasPoints> {
    mob: Mobject<T>,
    target: Mobject<T>,
    duration: f32,
}

impl<T: HasPoints> Deref for AnimationBuilder<T> {
    type Target = Mobject<T>;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

impl<T: HasPoints + Clone> AnimationBuilder<T> {
    pub fn new(mob: Mobject<T>, duration: f32) -> Self {
        Self {
            target: mob.clone(),
            mob,
            duration,
        }
    }
}

impl<T: HasPoints> Animatable for AnimationBuilder<T> {
    fn apply(&self, time: f32) -> bool {
        if time > self.duration {
            return false;
        }
        let progress = (time / self.duration).clamp(0.0, 1.0);
        self.mob.borrow_mut().interpolate(
            self.initial.as_ref().unwrap(),
            self.target.as_ref().unwrap(),
            self.easing.ease(progress),
        );
        true
    }

    fn begin(&mut self) {
        todo!()
    }
}
