use std::any::Any;

use cgmath::{Matrix4, One, Quaternion, Vector3, Vector4, Zero};


pub mod bezier;
pub mod shapes;
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::one(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
    fn get_matrix(&mut self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

pub trait Renderable {
    fn qbezier(&self) -> &bezier::QBezierPath;
    fn qbezier_mut(&mut self) -> &mut bezier::QBezierPath;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl<T: 'static> Renderable for Shape<T> {
    fn qbezier(&self) -> &bezier::QBezierPath {
        &self.qbezier
    }
    fn qbezier_mut(&mut self) -> &mut bezier::QBezierPath {
        &mut self.qbezier
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct Shape<T> {
    pub shape: T,
    pub qbezier: bezier::QBezierPath,
}

impl<T> Shape<T> {
    #[inline]
    pub fn rotate(&mut self, rotation: Quaternion<f32>) -> &mut Self {
        self.qbezier.rotate(rotation);
        self
    }
    #[inline]
    pub fn scale(&mut self, scale: impl Into<Vector3<f32>>) -> &mut Self {
        self.qbezier.scale(scale.into());
        self
    }
    #[inline]
    pub fn shift(&mut self, shift: impl Into<Vector3<f32>>) -> &mut Self {
        self.qbezier.shift(shift.into());
        self
    }
    #[inline]
    pub fn color(&mut self, color: impl Into<Vector4<f32>>) -> &mut Self {
        self.qbezier.color(color.into());
        self
    }
}
