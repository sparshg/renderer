use cgmath::Vector3;

use super::{bezier::QBezierPath, Shape};
use cgmath::{ElementWise, Quaternion, Vector4};
pub struct Square {
    side: f32,
    pub qbezier: QBezierPath,
}

impl Square {
    pub fn new(side: f32) -> Self {
        let points = [
            (1., 1., 0.),
            (0., 1., 0.),
            (-1., 1., 0.),
            (-1., 0., 0.),
            (-1., -1., 0.),
            (0., -1., 0.),
            (1., -1., 0.),
            (1., 0., 0.),
            (1., 1., 0.),
        ]
        .into_iter()
        .map(|(x, y, z)| Vector3::new(x, y, z) * side * 0.5)
        .collect();
        Self {
            side,
            qbezier: QBezierPath::new(points),
        }
    }
}

pub struct Circle {
    radius: f32,
    pub qbezier: QBezierPath,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        let angle = 2. * std::f32::consts::PI;
        let n_components = 8;
        let n_points = 2 * n_components + 1;
        let angles = (0..n_points).map(|i| i as f32 * angle / (n_points - 1) as f32);
        let mut points = angles
            .map(|angle| Vector3::new(angle.cos(), angle.sin(), 0.))
            .collect::<Vec<_>>();
        let theta = angle / n_components as f32;
        let handle_adjust = 1.0 / (theta / 2.0).cos();

        for i in (1..n_points).step_by(2) {
            points[i as usize] *= handle_adjust;
        }
        Self {
            radius,
            qbezier: QBezierPath::new(points),
        }
    }
}
