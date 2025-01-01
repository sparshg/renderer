use std::{any::Any, ops::Deref};

use cgmath::{ElementWise, Matrix4, One, Quaternion, Vector3, Vector4, VectorSpace, Zero};
use wgpu::{util::DeviceExt, CommandEncoder, ComputePipeline, RenderPipeline, ShaderStages};

use super::{
    utils::latch::Latch, AnyContext, Attach, BindGroupBuilder, ObjectUniforms, PipelineBuilder,
    SurfaceContext,
};

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
    pub fn get_matrix(&self) -> Matrix4<f32> {
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
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn update_render_buffers(&mut self, ctx: &SurfaceContext);
    fn update_compute_buffers(
        &mut self,
        ctx: &SurfaceContext,
        layout: &wgpu::BindGroupLayout,
    ) -> bool;
    fn num_compute_workgroups(&self) -> u32;
    fn get_render_object(&self) -> &RenderObject;
    fn get_compute_object(&self) -> &ComputeObject;
}

pub struct RenderObject {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    uniforms: Latch<ObjectUniforms>,
    //  renderer_type: PipelineType,
}
pub struct ComputeObject {
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
pub struct Shape<T> {
    shape: T,
    transform: Latch<Transform>,
    color: Latch<Vector4<f32>>,
    points: Latch<Vec<Vector3<f32>>>,
    render_object: Option<RenderObject>,
    compute_object: Option<ComputeObject>,
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
            transform: Latch::new(Transform::new()),
            points: Latch::new(points),
            color: Latch::new(Vector4::new(1.0, 1.0, 1.0, 1.0)),
            render_object: None,
            compute_object: None,
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
        *self.color = color.into();
        self
    }

    // pub fn interpolate<U, V>(&mut self, a: &Shape<U>, b: &Shape<V>, t: f32) {
    //     self.qbezier.points = a
    //         .qbezier
    //         .points
    //         .iter()
    //         .zip(b.qbezier.points.iter())
    //         .map(|(a, b)| a.lerp(*b, t))
    //         .collect();
    //     self.transform = a.transform.lerp(&b.transform, t);
    //     self.qbezier.uniforms = a.qbezier.uniforms.lerp(&b.qbezier.uniforms, t);
    //     self.update_uniforms = true;
    //     if let Some(ob) = self.qbezier.compute_object.as_mut() {
    //         ob.update = true;
    //     }
    // }
}

impl<T: 'static> Renderable for Shape<T> {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_render_object(&self) -> &RenderObject {
        self.render_object.as_ref().unwrap()
    }

    fn get_compute_object(&self) -> &ComputeObject {
        self.compute_object.as_ref().unwrap()
    }

    fn update_render_buffers(&mut self, ctx: &SurfaceContext) {
        let render_object = self.render_object.as_mut().unwrap();
        if self.transform.reset() {
            render_object.uniforms.model = self.transform.get_matrix();
        }
        if self.color.reset() {
            render_object.uniforms.color = *self.color;
        }
        if render_object.uniforms.reset() {
            let mut buff = encase::UniformBuffer::new(Vec::<u8>::new());
            buff.write(render_object.uniforms.deref()).unwrap();
            let buff = buff.into_inner();
            ctx.queue()
                .write_buffer(&render_object.uniform_buffer, 0, &buff);
        }
    }

    fn update_compute_buffers(
        &mut self,
        ctx: &SurfaceContext,
        layout: &wgpu::BindGroupLayout,
    ) -> bool {
        if !self.points.reset() {
            return false;
        }

        let mut data = encase::StorageBuffer::new(Vec::new());
        data.write(self.points.deref()).unwrap();
        let data: Vec<u8> = data.into_inner();

        let buffer = &self.compute_object.as_ref().unwrap().buffer;
        if (data.len() / 2..=data.len()).contains(&(buffer.size() as usize)) {
            ctx.queue().write_buffer(&buffer, 0, &data);
            return true;
        }

        let vertex_buffer = self.create_vertex_buffer(ctx);
        let index_buffer = self.create_index_buffer(ctx);

        let buffer = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Point Buffer"),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                contents: &data,
            });

        let bind_group = layout.attach(
            ctx,
            "Compute Bind Group",
            vec![
                buffer.as_entire_binding(),
                vertex_buffer.as_entire_binding(),
                index_buffer.as_entire_binding(),
            ],
        );

        let render_object = self.render_object.as_mut().unwrap();
        render_object.vertex_buffer = vertex_buffer;
        render_object.index_buffer = index_buffer;
        self.compute_object = Some(ComputeObject { bind_group, buffer });

        true
    }

    fn num_compute_workgroups(&self) -> u32 {
        (((self.points.len() / 2) as f32) / 64.0).ceil() as u32
    }
}

impl<T> Shape<T> {
    const VERTEX_SIZE: usize = 32;

    pub fn create_buffers(
        &mut self,
        ctx: &SurfaceContext,
        compute_layout: wgpu::BindGroupLayout,
        render_layout: wgpu::BindGroupLayout,
    ) {
        self.create_render_object(ctx, render_layout);
        self.create_compute_object(ctx, compute_layout);
    }

    fn create_vertex_buffer(&self, ctx: &SurfaceContext) -> wgpu::Buffer {
        ctx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (self.points.len() as u64 / 2 * 3 + 1) * Self::VERTEX_SIZE as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_index_buffer(&self, ctx: &SurfaceContext) -> wgpu::Buffer {
        ctx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (self.points.len() as u64 / 2 * 6)
                * std::mem::size_of::<u32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_render_object(&mut self, ctx: &SurfaceContext, layout: wgpu::BindGroupLayout) {
        let index_buffer = self.create_index_buffer(ctx);
        let vertex_buffer = self.create_vertex_buffer(ctx);
        let uniforms = Latch::new(ObjectUniforms::new(&self.transform, *self.color));

        let mut buff = encase::UniformBuffer::new(Vec::<u8>::new());
        buff.write(uniforms.deref()).unwrap();
        let buff = buff.into_inner();

        let uniform_buffer = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("QBezier Uniform Buffer"),
                contents: bytemuck::cast_slice(&buff),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = layout.attach(
            ctx,
            "QBezier Bind Group",
            vec![uniform_buffer.as_entire_binding()],
        );

        self.render_object = Some(RenderObject {
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            uniforms,
        });
    }

    fn create_compute_object(&mut self, ctx: &SurfaceContext, layout: wgpu::BindGroupLayout) {
        let mut data = encase::StorageBuffer::new(Vec::new());
        data.write(self.points.deref()).unwrap();
        let data: Vec<u8> = data.into_inner();

        let buffer = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Point Buffer"),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                contents: &data,
            });

        let render_object = self.render_object.as_ref().unwrap();

        let bind_group = layout.attach(
            ctx,
            "Compute Bind Group",
            vec![
                buffer.as_entire_binding(),
                render_object.vertex_buffer.as_entire_binding(),
                render_object.index_buffer.as_entire_binding(),
            ],
        );

        self.compute_object = Some(ComputeObject { bind_group, buffer });
    }
}
