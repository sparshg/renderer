pub mod easing;
use std::{cell::RefCell, ops::Deref, rc::Rc};

use cgmath::VectorSpace;
use easing::Easing;

use crate::core::{HasPoints, Mobject, Renderable, Shape};

pub trait Animation {
    fn apply(&self, time: f32);
    fn begin(&mut self);
    // fn get_target(&self) -> Rc<RefCell<dyn Renderable>>;
}

pub struct Transformation<T, V>
where
    T: HasPoints,
    V: HasPoints,
{
    duration: f32,
    mob: Rc<RefCell<Shape<T>>>,
    initial: Option<Shape<T>>,
    target: Option<Shape<V>>,
    initial_mob: Rc<RefCell<Shape<T>>>,
    target_mob: Rc<RefCell<Shape<V>>>,
    easing: Box<dyn Easing>,
}

impl<T, V> Transformation<T, V>
where
    T: HasPoints + Clone + 'static,
    V: HasPoints + Clone,
{
    pub fn new(initial: &Mobject<T>, target: &Mobject<V>, duration: f32) -> Self {
        Self {
            initial_mob: initial.deref().clone(),
            target_mob: target.deref().clone(),
            mob: initial.deref().clone(),
            initial: None,
            target: None,
            easing: Box::new(easing::Smooth),
            duration,
        }
    }
}

impl<T, V> Animation for Transformation<T, V>
where
    T: HasPoints + Clone + 'static,
    V: HasPoints + Clone,
{
    fn apply(&self, time: f32) {
        let progress = (time / self.duration).clamp(0.0, 1.0);
        self.mob.borrow_mut().interpolate(
            self.initial.as_ref().unwrap(),
            &self.target.as_ref().unwrap(),
            self.easing.ease(progress),
        );
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
