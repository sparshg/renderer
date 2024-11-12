use cgmath::{BaseFloat, Matrix4, Quaternion, Rad, Rotation3, SquareMatrix, Vector3};
use encase::ShaderType;
use wgpu::util::DeviceExt;

use crate::renderer::AnyContext;

struct Transform<T> {
    position: Vector3<T>,
    rotation: Quaternion<T>,
    scale: Vector3<T>,
}

impl<T: BaseFloat> From<Transform<T>> for Matrix4<T> {
    fn from(val: Transform<T>) -> Self {
        Matrix4::from_translation(val.position)
            * Matrix4::from(val.rotation)
            * Matrix4::from_nonuniform_scale(val.scale.x, val.scale.y, val.scale.z)
    }
}

trait Renderable {
    const VERTEX_SIZE_QBEZIER: usize = 32;

    fn update_buffers(
        &mut self,
        ctx: &impl AnyContext,
        vertex_buff: &mut Option<wgpu::Buffer>,
        index_buff: &mut Option<wgpu::Buffer>,
    );
    fn render(&self);
}
struct QBezier {
    points: Vec<Vector3<f32>>,
    point_buffer: Option<wgpu::Buffer>,
    compute_bgroup: Option<wgpu::BindGroup>,
}

impl Renderable for QBezier {
    fn update_buffers(
        &mut self,
        ctx: &impl AnyContext,
        vertex_buff: &mut Option<wgpu::Buffer>,
        index_buff: &mut Option<wgpu::Buffer>,
    ) {
        let mut data = encase::StorageBuffer::new(Vec::new());
        data.write(&self.points).unwrap();
        let data: Vec<u8> = data.into_inner();

        if let Some(buff) = self.point_buffer.as_ref() {
            ctx.queue().write_buffer(buff, 0, &data);
        } else {
            self.point_buffer = Some(ctx.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Point Buffer"),
                    usage: wgpu::BufferUsages::STORAGE,
                    contents: &data,
                },
            ))
        }

        if vertex_buff.is_none() {
            *vertex_buff = Some(ctx.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (self.points.len() as u64 / 2 * 3 + 1)
                    * Self::VERTEX_SIZE_QBEZIER as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        if index_buff.is_none() {
            *index_buff = Some(ctx.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (self.points.len() as u64 / 2 * 6)
                    * std::mem::size_of::<u32>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        // self.compute_bgroup = Some(ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: None,
        //     layout: &bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: point_buff.as_entire_binding(),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: vert_buff.as_entire_binding(),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 2,
        //             resource: ind_buff.as_entire_binding(),
        //         },
        //     ],
        // }));
    }
    fn render(&self) {}
}

struct Mesh {
    vertices: Vec<Vector3<f32>>,
    indices: Vec<u32>,
}

#[derive(ShaderType)]
struct ObjectUniforms {
    model: Matrix4<f32>,
    color: Vector3<f32>,
}

struct Object<T: Renderable> {
    transform: Transform<f32>,
    uniforms: ObjectUniforms,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
    renderable: T,
}

impl Object<QBezier> {
    fn new(points: Vec<Vector3<f32>>) -> Self {
        Self {
            transform: Transform {
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: Quaternion::from_angle_x(Rad(0.0)),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
            uniforms: ObjectUniforms {
                model: Matrix4::identity(),
                color: Vector3::new(1.0, 1.0, 1.0),
            },
            vertex_buffer: None,
            index_buffer: None,
            bind_group: None,
            renderable: QBezier {
                points,
                point_buffer: None,
                compute_bgroup: todo!(),
            },
        }
    }
    fn update_points(&mut self, ctx: &impl AnyContext, points: Vec<Vector3<f32>>) {
        self.renderable.points = points;
        self.renderable
            .update_buffers(ctx, &mut self.vertex_buffer, &mut self.index_buffer);
    }
}
