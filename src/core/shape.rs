use std::{
    any::Any,
    cell::{Cell, RefCell},
    hash::Hash,
    ops::Deref,
    rc::Rc,
};

use cgmath::{ElementWise, Matrix4, One, Quaternion, Vector3, Vector4, VectorSpace, Zero};
use wgpu::util::DeviceExt;

use super::{utils::latch::Latch, AnyContext, Attach, ObjectUniforms, SurfaceContext};

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
    // fn as_any_mut(&mut self) -> &mut dyn Any;
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
}
pub struct ComputeObject {
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mobject<T: HasPoints> {
    inner: Rc<RefCell<Shape<T>>>,
}

impl<T: HasPoints> Deref for Mobject<T> {
    type Target = Rc<RefCell<Shape<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: HasPoints> Mobject<T> {
    pub fn new(mobject: Shape<T>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(mobject)),
        }
    }

    pub fn rotate(&self, rotation: Quaternion<f32>) -> &Self {
        self.borrow_mut().transform.rotation = rotation * self.borrow_mut().transform.rotation;
        self
    }
    pub fn scale_vec(&self, scale: impl Into<Vector3<f32>>) -> &Self {
        self.borrow_mut()
            .transform
            .scale
            .mul_assign_element_wise(scale.into());
        self
    }
    pub fn scale(&self, scale: f32) -> &Self {
        self.borrow_mut()
            .transform
            .scale
            .mul_assign_element_wise(Vector3::new(scale, scale, scale));
        self
    }
    pub fn shift(&self, offset: impl Into<Vector3<f32>>) -> &Self {
        self.borrow_mut().transform.position += offset.into();
        self
    }

    pub fn color(&self, color: impl Into<Vector4<f32>>) -> &Self {
        *self.borrow_mut().color = color.into();
        self
    }

    // pub fn interpolate<U, V>(&self, a: &Shape<U>, b: &Shape<V>, t: f32) {
    //     let mut self_mut = self.borrow_mut();
    //     *self_mut.points = a
    //         .points
    //         .iter()
    //         .zip(b.points.iter())
    //         .map(|(a, b)| a.lerp(*b, t))
    //         .collect();
    //     *self_mut.transform = a.transform.lerp(&b.transform, t);
    //     *self_mut.render_object.as_mut().unwrap().uniforms = a
    //         .render_object
    //         .as_ref()
    //         .unwrap()
    //         .uniforms
    //         .lerp(&b.render_object.as_ref().unwrap().uniforms, t);
    // }
}

pub trait HasPoints {
    fn calc_points(&self) -> Vec<Vector3<f32>>;
}

pub struct Shape<T: HasPoints> {
    shape: Latch<T>,
    transform: Latch<Transform>,
    color: Latch<Vector4<f32>>,
    pub points: Vec<Vector3<f32>>,
    render_object: Option<RenderObject>,
    compute_object: Option<ComputeObject>,
}

impl<T> Clone for Shape<T>
where
    T: Clone + HasPoints,
{
    fn clone(&self) -> Self {
        Self {
            shape: self.shape.clone(),
            transform: self.transform.clone(),
            color: self.color.clone(),
            points: self.points.clone(),
            render_object: None,
            compute_object: None,
        }
    }
}

impl<T: HasPoints> Deref for Shape<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.shape
    }
}

impl<T: HasPoints> Shape<T> {
    pub fn new(shape: T) -> Self {
        Self {
            shape: Latch::new_set(shape),
            transform: Latch::new_reset(Transform::new()),
            color: Latch::new_reset(Vector4::new(1.0, 1.0, 1.0, 1.0)),
            points: Vec::new(),
            render_object: None,
            compute_object: None,
        }
    }
}

impl<T: HasPoints + 'static> Renderable for Shape<T> {
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
        if !self.shape.reset() {
            return false;
        }
        self.points = self.shape.calc_points();

        let mut data = encase::StorageBuffer::new(Vec::new());
        data.write(self.points.deref()).unwrap();
        let data: Vec<u8> = data.into_inner();

        if self
            .compute_object
            .as_ref()
            .is_some_and(|ob| (data.len() / 2..=data.len()).contains(&(ob.buffer.size() as usize)))
        {
            ctx.queue()
                .write_buffer(&self.compute_object.as_ref().unwrap().buffer, 0, &data);
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

impl<T: HasPoints> Shape<T> {
    const VERTEX_SIZE: usize = 32;

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

    pub fn create_render_object(&mut self, ctx: &SurfaceContext, layout: wgpu::BindGroupLayout) {
        self.points = self.shape.calc_points();
        let index_buffer = self.create_index_buffer(ctx);
        let vertex_buffer = self.create_vertex_buffer(ctx);
        let uniforms = Latch::new_reset(ObjectUniforms::new(&self.transform, *self.color));

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
}
