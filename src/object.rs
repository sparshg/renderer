use cgmath::{
    BaseFloat, ElementWise, Matrix4, Quaternion, Rad, Rotation3, SquareMatrix, Vector3, Vector4,
};
use encase::ShaderType;
use wgpu::{util::DeviceExt, BufferAddress};

use crate::renderer::{AnyContext, Attach, Renderable};

struct Transform<T> {
    position: Vector3<T>,
    rotation: Quaternion<T>,
    scale: Vector3<T>,
}

impl<T: BaseFloat> Transform<T> {
    fn new() -> Self {
        Self {
            position: Vector3::new(T::zero(), T::zero(), T::zero()),
            rotation: Quaternion::from_angle_x(Rad(T::zero())),
            scale: Vector3::new(T::one(), T::one(), T::one()),
        }
    }
    fn get_matrix(&mut self) -> Matrix4<T> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

pub struct QBezier {
    points: Vec<Vector3<f32>>,
    point_buffer: Option<wgpu::Buffer>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    compute_bgroup: Option<wgpu::BindGroup>,
    update_points: bool,
    update_uniforms: bool,
    transform: Transform<f32>,
    uniforms: ObjectUniforms,
    uniforms_buffer: Option<wgpu::Buffer>,
    render_bgroup: Option<wgpu::BindGroup>,
}

impl Renderable for QBezier {
    const VERTEX_SIZE: usize = 32;

    fn update_render_buffers(&mut self, ctx: &impl AnyContext, layout: &wgpu::BindGroupLayout) {
        if !self.update_uniforms {
            return;
        }
        self.update_uniforms = false;
        self.uniforms.model = self.transform.get_matrix();

        let mut data = encase::UniformBuffer::new(Vec::new());
        data.write(&self.uniforms).unwrap();
        let data = data.into_inner();

        match self.uniforms_buffer {
            Some(ref b) => {
                ctx.queue().write_buffer(b, 0, &data);
            }
            None => {
                self.uniforms_buffer = Some(ctx.device().create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Uniform Buffer"),
                        contents: &data,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    },
                ));
            }
        };
        self.render_bgroup.get_or_insert_with(|| {
            layout.attach(
                ctx,
                "Render Bind Group Uniforms",
                vec![self.uniforms_buffer.as_ref().unwrap().as_entire_binding()],
            )
        });
    }

    fn update_compute_buffers(
        &mut self,
        ctx: &impl AnyContext,
        layout: &wgpu::BindGroupLayout,
    ) -> bool {
        if !self.update_points {
            return false;
        }
        self.update_points = false;
        let mut data = encase::StorageBuffer::new(Vec::new());
        data.write(&self.points).unwrap();
        let data: Vec<u8> = data.into_inner();

        let mut reinit_buffers = false;
        match self.point_buffer {
            Some(ref b) if b.size() == data.len() as BufferAddress => {
                ctx.queue()
                    .write_buffer(self.point_buffer.as_ref().unwrap(), 0, &data);
            }
            _ => {
                reinit_buffers = true;
                self.point_buffer = Some(ctx.device().create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Point Buffer"),
                        usage: wgpu::BufferUsages::STORAGE,
                        contents: &data,
                    },
                ));
            }
        }

        if reinit_buffers {
            self.vertex_buffer = Some(ctx.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (self.points.len() as u64 / 2 * 3 + 1)
                    * Self::VERTEX_SIZE as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        if reinit_buffers {
            self.index_buffer = Some(ctx.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (self.points.len() as u64 / 2 * 6)
                    * std::mem::size_of::<u32>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        if reinit_buffers {
            self.compute_bgroup = Some(layout.attach(
                ctx,
                "Compute Bind Group",
                vec![
                    self.point_buffer.as_ref().unwrap().as_entire_binding(),
                    self.vertex_buffer.as_ref().unwrap().as_entire_binding(),
                    self.index_buffer.as_ref().unwrap().as_entire_binding(),
                ],
            ));
        }
        true
    }

    fn get_compute_bgroup(&self) -> &wgpu::BindGroup {
        self.compute_bgroup.as_ref().unwrap()
    }

    fn num_compute_workgroups(&self) -> u32 {
        (((self.points.len() / 2) as f32) / 64.0).ceil() as u32
    }

    fn get_index_buffer(&self) -> &wgpu::Buffer {
        self.index_buffer.as_ref().unwrap()
    }

    fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        self.vertex_buffer.as_ref().unwrap()
    }

    fn get_render_bgroup(&self) -> &wgpu::BindGroup {
        self.render_bgroup.as_ref().unwrap()
    }
}

impl QBezier {
    pub fn new(points: Vec<Vector3<f32>>) -> Self {
        Self {
            points,
            point_buffer: None,
            vertex_buffer: None,
            index_buffer: None,
            compute_bgroup: None,
            uniforms_buffer: None,
            render_bgroup: None,
            update_points: true,
            update_uniforms: true,
            transform: Transform::new(),
            uniforms: ObjectUniforms {
                model: Matrix4::identity(),
                color: Vector4::new(0.0, 1.0, 1.0, 1.0),
            },
        }
    }

    pub fn shift(&mut self, translation: Vector3<f32>) {
        self.transform.position += translation;
        self.update_uniforms = true;
    }

    pub fn rotate(&mut self, rotation: Quaternion<f32>) {
        self.transform.rotation = rotation * self.transform.rotation;
        self.update_uniforms = true;
    }

    pub fn scale(&mut self, scale: Vector3<f32>) {
        self.transform.scale.mul_assign_element_wise(scale);
        self.update_uniforms = true;
    }

    pub fn color(&mut self, color: Vector4<f32>) {
        self.uniforms.color = color;
    }
}

struct Mesh {
    vertices: Vec<Vector3<f32>>,
    indices: Vec<u32>,
}

#[derive(Debug, ShaderType)]
struct ObjectUniforms {
    model: Matrix4<f32>,
    color: Vector4<f32>,
}

// struct Object<T: Renderable> {
//     renderable: T,
// }

// impl Object<QBezier> {
//     fn new(points: Vec<Vector3<f32>>) -> Self {
//         Self {
//             renderable: QBezier {
//                 points,
//                 point_buffer: None,
//                 vertex_buffer: None,
//                 index_buffer: None,
//                 compute_bgroup: None,
//                 uniforms_buffer: None,
//                 render_bgroup: None,
//                 update_points: true,
//                 transform: Transform::new(),
//                 uniforms: ObjectUniforms {
//                     model: Matrix4::identity(),
//                     color: Vector4::new(1.0, 1.0, 1.0, 1.0),
//                 },
//             },
//         }
//     }
//     fn update_points(&mut self, ctx: &impl AnyContext, points: Vec<Vector3<f32>>) {
//         self.renderable.points = points;
//         self.renderable.update_buffers(ctx);
//     }
// }
