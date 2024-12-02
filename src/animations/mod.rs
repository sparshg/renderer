use cgmath::VectorSpace;

use crate::geometry::bezier::QBezierPath;

pub struct Animation {
    src: QBezierPath,
    dst: QBezierPath,
    cur: QBezierPath,
    dur: f32,
}

impl Animation {
    pub fn new(src: &QBezierPath, dst: &QBezierPath, dur: f32) -> Self {
        Self {
            cur: src.clone(),
            src: src.clone(),
            dst: dst.clone(),
            dur,
        }
    }

    pub fn update(&mut self, t: f32) {
        let t = t / self.dur;
        self.cur = QBezierPath {
            points: self
                .src
                .points
                .iter()
                .zip(self.dst.points.iter())
                .map(|(a, b)| a.lerp(*b, t))
                .collect(),
            render_object: None,
            compute_object: None,
        }
    }
}
