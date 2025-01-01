// use cgmath::VectorSpace;

// use crate::core::{Renderable, Shape};

// pub trait Animation {
//     fn apply(&mut self, time: f32);
//     fn get_target(&mut self) -> &mut Box<dyn Renderable>;
// }

// pub struct Transformation<T, V> {
//     duration: f32,
//     pub curr: Box<dyn Renderable>,
//     from: Shape<T>,
//     to: Shape<V>,
// }

// impl<T, V> Transformation<T, V>
// where
//     T: Clone + 'static,
//     V: Clone,
// {
//     pub fn new(from: &Shape<T>, to: &Shape<V>, duration: f32) -> Self {
//         let curr = from.clone();
//         let mut from = curr.clone();

//         let mut to = to.clone();
//         let max_len = from.qbezier.points.len().max(to.qbezier.points.len());

//         let len = from.qbezier.points.len();
//         if len < max_len {
//             let ex: Vec<_> = from
//                 .qbezier
//                 .points
//                 .iter()
//                 .cycle()
//                 .take(max_len - len)
//                 .cloned()
//                 .collect();
//             from.qbezier.points.extend(ex);
//         }

//         let len = to.qbezier.points.len();
//         if len < max_len {
//             let ex: Vec<_> = to
//                 .qbezier
//                 .points
//                 .iter()
//                 .cycle()
//                 .take(max_len - len)
//                 .cloned()
//                 .collect();
//             to.qbezier.points.extend(ex);
//         }

//         Self {
//             to,
//             from,
//             curr: Box::new(curr),
//             duration,
//         }
//     }
// }

// impl<T, V> Animation for Transformation<T, V>
// where
//     T: Clone + 'static,
//     V: Clone,
// {
//     fn apply(&mut self, time: f32) {
//         let progress = (time / self.duration).clamp(0.0, 1.0);
//         self.curr
//             .as_any_mut()
//             .downcast_mut::<Shape<T>>()
//             .unwrap()
//             .interpolate(&self.from, &self.to, progress);
//     }

//     fn get_target(&mut self) -> &mut Box<dyn Renderable> {
//         &mut self.curr
//     }
// }
