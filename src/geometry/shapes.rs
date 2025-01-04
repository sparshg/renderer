use cgmath::Vector3;

use crate::core::{HasPoints, Mobject, Shape};

#[derive(Clone)]
pub struct Square {
    side: f32,
}

impl Square {
    pub fn new(side: f32) -> Mobject<Square> {
        Mobject::new(Shape::new(Self { side }))
    }
}

impl HasPoints for Square {
    fn calc_points(&self) -> Vec<Vector3<f32>> {
        [
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
        .map(|(x, y, z)| Vector3::new(x, y, z) * self.side * 0.5)
        .collect::<Vec<_>>()
    }
}

#[derive(Clone)]
pub struct Arc {
    pub radius: f32,
    angle: f32,
}

impl Arc {
    pub fn new(radius: f32, angle: f32) -> Mobject<Arc> {
        Mobject::new(Shape::new(Arc { radius, angle }))
    }

    pub fn circle(radius: f32) -> Mobject<Arc> {
        Self::new(radius, std::f32::consts::PI * 2.)
    }
}

impl HasPoints for Arc {
    fn calc_points(&self) -> Vec<Vector3<f32>> {
        let n_components = 8;
        let n_points = 2 * n_components + 1;
        let angles = (0..n_points).map(|i| i as f32 * self.angle / (n_points - 1) as f32);
        let mut points = angles
            .map(|angle| Vector3::new(angle.cos(), angle.sin(), 0.) * self.radius)
            .collect::<Vec<_>>();
        let theta = self.angle / n_components as f32;
        let handle_adjust = 1.0 / (theta / 2.0).cos();

        for i in (1..n_points).step_by(2) {
            points[i as usize] *= handle_adjust;
        }
        points
    }
}
