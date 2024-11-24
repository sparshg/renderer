
use cgmath::{Matrix4, One, Quaternion, Vector3, Vector4, Zero};

use crate::renderer::SurfaceContext;

pub mod bezier;

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

pub trait ShapeBuffer: Shape {
    fn create_render_buffers(&mut self, ctx: &SurfaceContext, layout: &wgpu::BindGroupLayout);
    fn update_compute_buffers(
        &mut self,
        ctx: &SurfaceContext,
        layout: &wgpu::BindGroupLayout,
    ) -> bool;
    fn update_render_buffers(&mut self, ctx: &SurfaceContext);
    fn num_compute_workgroups(&self) -> u32;
}

pub trait Shape {
    // fn get_render_object(&self) -> Weak<RefCell<RenderObject>>;
    fn shift(&mut self, offset: Vector3<f32>);
    fn rotate(&mut self, rotation: Quaternion<f32>);
    fn scale(&mut self, scale: Vector3<f32>);
    fn color(&mut self, color: Vector4<f32>);
    // fn bounds(&self) -> (Vector3<f32>, Vector3<f32>); // min, max
}

pub trait Path: Shape {
    fn points(&self) -> &[Vector3<f32>];
}

pub trait Mesh: Shape {
    fn vertices(&self) -> &[Vector3<f32>];
    fn normals(&self) -> &[Vector3<f32>];
    fn indices(&self) -> &[u32];
}
