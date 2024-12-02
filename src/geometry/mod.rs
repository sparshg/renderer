use std::{any::Any, ops::Deref};

use cgmath::{ElementWise, Matrix4, One, Quaternion, Vector3, Vector4, VectorSpace, Zero};

use crate::renderer::ObjectUniforms;

pub mod bezier;
pub mod shapes;

#[derive(Clone)]
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
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

pub trait Renderable {
    fn qbezier(&self) -> &bezier::QBezierPath;
    fn qbezier_mut(&mut self) -> &mut bezier::QBezierPath;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn update_uniforms(&mut self);
    fn uniforms(&self) -> &ObjectUniforms;
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
    fn update_uniforms(&mut self) {
        // TODO: recompute only if needed
        self.uniform.model = self.transform.get_matrix();
    }
    fn uniforms(&self) -> &ObjectUniforms {
        &self.uniform
    }
}

pub struct Shape<T> {
    shape: T,
    transform: Transform,
    uniform: ObjectUniforms,
    qbezier: bezier::QBezierPath,
}

impl<T> Deref for Shape<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.shape
    }
}

impl<T> Shape<T> {
    pub fn new(shape: T, points: Vec<Vector3<f32>>) -> Self {
        Self {
            shape,
            transform: Transform::new(),
            uniform: ObjectUniforms::default(),
            qbezier: bezier::QBezierPath::new(points),
        }
    }
    pub fn rotate(&mut self, rotation: Quaternion<f32>) -> &mut Self {
        self.transform.rotation = rotation * self.transform.rotation;
        self
    }
    pub fn scale_vec(&mut self, scale: impl Into<Vector3<f32>>) -> &mut Self {
        self.transform.scale.mul_assign_element_wise(scale.into());
        self
    }
    pub fn scale(&mut self, scale: f32) -> &mut Self {
        self.transform
            .scale
            .mul_assign_element_wise(Vector3::new(scale, scale, scale));
        self
    }
    pub fn shift(&mut self, offset: impl Into<Vector3<f32>>) -> &mut Self {
        self.transform.position += offset.into();
        self
    }
    pub fn color(&mut self, color: impl Into<Vector4<f32>>) -> &mut Self {
        self.uniform.color = color.into();
        self
    }
}
