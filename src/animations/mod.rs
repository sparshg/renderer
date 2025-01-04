use std::{cell::RefCell, ops::Deref, rc::Rc};

use cgmath::VectorSpace;

use crate::core::{HasPoints, Mobject, Renderable, Shape};

pub trait Animation {
    fn apply(&self, time: f32);
    // fn get_target(&self) -> Rc<RefCell<dyn Renderable>>;
}

pub struct Transformation<T, V>
where
    T: HasPoints,
    V: HasPoints,
{
    duration: f32,
    mob: Rc<RefCell<Shape<T>>>,
    initial: Shape<T>,
    target: Shape<V>,
}

impl<T, V> Transformation<T, V>
where
    T: HasPoints + Clone + 'static,
    V: HasPoints + Clone,
{
    // TODO: Clone points every time?
    pub fn new(initial: &Mobject<T>, target: &Mobject<V>, duration: f32) -> Self {
        let mob = initial.deref().clone();
        let mut initial = initial.deref().borrow().clone();
        let mut target = target.deref().borrow().clone();

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

        Self {
            initial,
            target,
            mob,
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
        self.mob
            .borrow_mut()
            .interpolate(&self.initial, &self.target, progress);
    }

    // fn get_target(&self) -> Rc<RefCell<dyn Renderable>> {
    //     self.mob
    // }
}
