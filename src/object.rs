use cgmath::{BaseFloat, Matrix4, Quaternion, Rad, Rotation3, SquareMatrix, Vector3};
use encase::ShaderType;
use wgpu::{util::DeviceExt, ComputePipeline, RenderPipeline};

use crate::renderer::{AnyContext, Attach, BindGroupBuilder, Pipeline, PipelinePass, Renderable};

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

pub struct QBezier {
    points: Vec<Vector3<f32>>,
    point_buffer: Option<wgpu::Buffer>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    compute_bgroup: Option<wgpu::BindGroup>,
    render_bgroup: Option<wgpu::BindGroup>,
    reinit_buffers: bool,
}

impl Renderable for QBezier {
    const VERTEX_SIZE: usize = 32;

    // fn get_render_bg_layout(&mut self, ctx: &impl AnyContext) -> &wgpu::BindGroupLayout {
    //     self.render_bg_layout.get_or_insert_with(|| {
    //         BindGroupBuilder::new("QBezier Render Bind Group layout")
    //             .add_uniform_buffer(wgpu::ShaderStages::VERTEX, None)
    //             // TODO: min_binding_size is None everywhere
    //             .build(ctx)
    //     })
    // }

    fn update_buffers(&mut self, ctx: &impl AnyContext) {
        let mut data = encase::StorageBuffer::new(Vec::new());
        data.write(&self.points).unwrap();
        let data: Vec<u8> = data.into_inner();

        if self.point_buffer.is_none()
            || data.len() != self.point_buffer.as_ref().unwrap().size() as usize
        // TODO: Only recreate when size is increasing?
        {
            self.reinit_buffers = true;
            self.point_buffer = Some(ctx.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Point Buffer"),
                    usage: wgpu::BufferUsages::STORAGE,
                    contents: &data,
                },
            ));
        } else {
            ctx.queue()
                .write_buffer(self.point_buffer.as_ref().unwrap(), 0, &data);
        }

        if self.reinit_buffers {
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

        if self.reinit_buffers {
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
    }

    fn compute(
        &mut self,
        ctx: &impl AnyContext,
        pipeline_pass: PipelinePass<'_, ComputePipeline>,
        layout: &wgpu::BindGroupLayout,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if self.reinit_buffers {
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
        self.reinit_buffers = false;

        pipeline_pass
            .add_bind_group(self.compute_bgroup.as_ref().unwrap())
            .pass(
                encoder,
                (
                    (((self.points.len() / 2) as f32) / 64.0).ceil() as u32,
                    1,
                    1,
                ),
            );
    }

    fn render(&self, pipeline: &[Pipeline<RenderPipeline>], encoder: &mut wgpu::CommandEncoder) {
        pipeline[0]
            .begin_pass("Stencil Pass")
            .add_bind_group(&self.camera.bind_group)
            .add_vertex_buffer(&self.vertex_buffer.unwrap())
            .add_index_buffer(&self.index_buffer.unwrap())
            .pass(
                &mut encoder,
                &[],
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: None,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
            );
        pipeline[1]
            .begin_pass("Render Pass")
            .add_bind_group(&self.camera.bind_group)
            .add_vertex_buffer(&self.vertex_buffer.unwrap())
            .add_index_buffer(&self.index_buffer.unwrap())
            .set_stencil_reference(1)
            .pass(
                &mut encoder,
                &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            );
    }
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
        self.renderable.update_buffers(ctx);
    }
}
