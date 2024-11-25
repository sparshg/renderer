use cgmath::{ElementWise, Matrix4, One, Quaternion, Vector3, Vector4, Zero};

use crate::renderer::SurfaceContext;

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

pub trait ShapeBuffer {
    fn create_render_buffers(&mut self, ctx: &SurfaceContext, layout: &wgpu::BindGroupLayout);
    fn update_compute_buffers(
        &mut self,
        ctx: &SurfaceContext,
        layout: &wgpu::BindGroupLayout,
    ) -> bool;
    fn update_render_buffers(&mut self, ctx: &SurfaceContext);
    fn num_compute_workgroups(&self) -> u32;
}

pub enum Shape {
    Square(shapes::Square),
    Circle(shapes::Circle),
}

macro_rules! match_variants {
    ($enum:expr, $field:ident, mut) => {
        match $enum {
            Self::Square(s) => &mut s.$field,
            Self::Circle(s) => &mut s.$field,
        }
    };
    ($enum:expr, $field:ident, ref) => {
        match $enum {
            Self::Square(s) => &s.$field,
            Self::Circle(s) => &s.$field,
        }
    };
    ($enum:expr, $field:ident, $method:ident $(, $args:expr)*) => {
        match $enum {
            Self::Square(s) => s.$field.$method($($args),*),
            Self::Circle(s) => s.$field.$method($($args),*),
        }
    };
}

impl Shape {
    pub fn square(side: f32) -> Self {
        Self::Square(shapes::Square::new(side))
    }
    pub fn circle(radius: f32) -> Self {
        Self::Circle(shapes::Circle::new(radius))
    }
    pub fn rotate(&mut self, rotation: Quaternion<f32>) -> &mut Self {
        match_variants!(self, qbezier, rotate, rotation);
        self
    }
    pub fn scale(&mut self, scale: impl Into<Vector3<f32>>) -> &mut Self {
        match_variants!(self, qbezier, scale, scale.into());
        self
    }
    pub fn shift(&mut self, shift: impl Into<Vector3<f32>>) -> &mut Self {
        match_variants!(self, qbezier, shift, shift.into());
        self
    }
    pub fn color(&mut self, color: impl Into<Vector4<f32>>) -> &mut Self {
        match_variants!(self, qbezier, color, color.into());
        self
    }
    pub fn qbezier_ref(&self) -> &bezier::QBezierPath {
        match_variants!(self, qbezier, ref)
    }
    pub fn qbezier_mut(&mut self) -> &mut bezier::QBezierPath {
        match_variants!(self, qbezier, mut)
    }
    pub fn get_type(&self) -> Type {
        match self {
            Self::Square(_) => Type::Square,
            Self::Circle(_) => Type::Circle,
        }
    }
}
