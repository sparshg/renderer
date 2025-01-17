pub mod anim;
pub mod builder;
pub mod easing;
use std::{cell::RefCell, ops::Deref, rc::Rc};

use easing::Easing;

use crate::core::{HasPoints, Mobject, Shape};

pub trait Animatable {
    fn apply(&self, time: f32) -> bool;
    fn begin(&mut self);
    // fn get_target(&self) -> Rc<RefCell<dyn Renderable>>;
}

pub struct Transformation<T, V>
where
    T: HasPoints,
    V: HasPoints,
{
    duration: f32,
    mob: Mobject<T>,
    initial: Option<Shape<T>>,
    target: Option<Shape<V>>,
    initial_mob: Mobject<T>,
    target_mob: Mobject<V>,
    easing: Box<dyn Easing>,
}

impl<T, V> Transformation<T, V>
where
    T: HasPoints + Clone,
    V: HasPoints + Clone,
{
    pub fn new(initial: &Mobject<T>, target: &Mobject<V>, duration: f32) -> Self {
        Self {
            initial_mob: initial.ref_clone(),
            target_mob: target.ref_clone(),
            mob: initial.ref_clone(),
            initial: None,
            target: None,
            easing: Box::new(easing::Smooth),
            duration,
        }
    }
}

impl<T, V> Animatable for Transformation<T, V>
where
    T: HasPoints + Clone,
    V: HasPoints + Clone,
{
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
    // TODO: Clone points every time?

    fn begin(&mut self) {
        let mut initial = self.initial_mob.deref().borrow().clone();
        let mut target = self.target_mob.deref().borrow().clone();
        if initial.points.len() == 0 {
            *initial.points = initial.calc_points();
        }

        if target.points.len() == 0 {
            *target.points = target.calc_points();
        }

        let max_len = initial.points.len().max(target.points.len());

        let len = initial.points.len();
        if len < max_len {
            *initial.points = (0..max_len)
                .map(|i| initial.points[i * len / max_len])
                .collect();
        }

        let len = target.points.len();
        if len < max_len {
            *target.points = (0..max_len)
                .map(|i| target.points[i * len / max_len])
                .collect();
        }
        self.initial = Some(initial);
        self.target = Some(target);
    }

    // fn get_target(&self) -> Rc<RefCell<dyn Renderable>> {
    //     self.mob
    // }
}
