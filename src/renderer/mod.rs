mod camera;
mod utils;
mod window;
use core::panic;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use camera::Camera;
use cgmath::Matrix4;
use cgmath::SquareMatrix;
use cgmath::Vector4;
use cgmath::VectorSpace;
use encase::ShaderType;
pub use utils::bindgroup::{Attach, BindGroupBuilder};
pub use utils::context::AnyContext;
pub use utils::context::Context;
pub use utils::context::SurfaceContext;
use utils::pipeline::IntoPass;
pub use utils::pipeline::PipelineBuilder;
use wgpu::CommandEncoder;
use wgpu::ComputePipeline;
use wgpu::RenderPipeline;
use wgpu::ShaderStages;
pub use window::App;
pub use window::Window;

use crate::animations::Animation;
use crate::geometry::bezier::QBezierPath;
use crate::geometry::Renderable;
use crate::geometry::Shape;
use crate::texture::Texture;

// pub enum PipelineType {
//     QBezier,
//     // Mesh,
// }

#[derive(Debug, ShaderType, Clone)]
pub struct ObjectUniforms {
    pub model: Matrix4<f32>,
    pub color: Vector4<f32>,
}

impl Default for ObjectUniforms {
    fn default() -> Self {
        Self {
            model: Matrix4::identity(),
            color: Vector4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl ObjectUniforms {
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            model: self.model.lerp(other.model, t),
            color: self.color.lerp(other.color, t),
        }
    }
}

pub struct RenderObject {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    // pub renderer_type: PipelineType,
}
pub struct ComputeObject {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub update: bool,
}

#[derive(Clone)]
pub struct Id<T: 'static> {
    pub id: u32,
    _marker: std::marker::PhantomData<T>,
}

pub struct Scene {
    // ctx: &'a SurfaceContext<'a>,
    pub camera: Camera,
    pub depth_texture: Texture,
    pub objects: HashMap<u32, Box<dyn Renderable>>,
    pub animations: Vec<Box<dyn Animation>>,
    pub qbezier_renderer: QBezierRenderer,
    t: f32,
    // mesh_renderer: MeshRenderer,
}

#[macro_export]
macro_rules! add {
    ($scene:ident, $ctx:ident, $($shape:ident),*) => {
        $(
            let $shape = $scene.add($ctx, $shape);
        )*
    };
}

#[macro_export]
macro_rules! remove {
    ($scene:ident, $($shape:ident),*) => {
        $(
            $scene.remove($shape);
        )*
    };
}

impl Scene {
    pub fn new(ctx: &SurfaceContext) -> Self {
        let depth_texture = Texture::create_depth_texture(
            &ctx.device,
            (ctx.config.width, ctx.config.height),
            "Depth Texture",
        );
        let camera = Camera::new(ctx);
        Self {
            depth_texture,
            objects: HashMap::new(),
            qbezier_renderer: QBezierRenderer::new(ctx, &camera.bind_group_layout),
            camera,
            animations: Vec::new(),
            t: 0.0,
        }
    }

    pub fn add<T: 'static>(&mut self, ctx: &SurfaceContext, mut shape: Shape<T>) -> Id<T> {
        shape.qbezier_mut().create_render_buffers(
            ctx,
            &self
                .qbezier_renderer
                .render_pipeline
                .get_bind_group_layout(1),
        );
        let id = self.objects.len() as u32;
        self.objects.insert(id, Box::new(shape));
        Id {
            id,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn remove<T: 'static>(&mut self, id: Id<T>) {
        self.objects.remove(&id.id);
    }

    pub fn update(&mut self, ctx: &SurfaceContext) {
        self.camera.update_camera(ctx);
        for anim in self.animations.iter_mut() {
            anim.apply(self.t);
            println!("{}", self.t);
            self.t += self.t * 0.008 + 0.01;
        }
        if self.t > 1. {
            dbg!(&self.animations[0].get_target().qbezier().points);
            panic!();
        }
    }

    pub fn render(&mut self, ctx: &SurfaceContext, view: &wgpu::TextureView) {
        // self.objects.retain(|_, object| object.upgrade().is_some());
        let mut encoder = ctx.device().create_command_encoder(&Default::default());

        for object in self.objects.values_mut() {
            // match object.renderer_type {
            // PipelineType::QBezier => {
            // let ob
            self.qbezier_renderer.render(
                ctx,
                view,
                &self.depth_texture.view,
                &self.camera.bind_group,
                &mut encoder,
                object,
                false,
            );
            // }
            // PipelineType::Mesh => {
            //     self.mesh_renderer
            //         .render(ctx, view, &mut encoder, object, false);
            // }
            // }
        }

        for anim in self.animations.iter_mut() {
            let mut object = anim.get_target();
            self.qbezier_renderer.render(
                ctx,
                view,
                &self.depth_texture.view,
                &self.camera.bind_group,
                &mut encoder,
                object,
                false,
            );
        }

        ctx.queue().submit(std::iter::once(encoder.finish()));
    }

    pub fn modify<T: 'static>(&mut self, id: &Id<T>, f: impl FnOnce(&mut Shape<T>)) {
        let ob = self.objects.get_mut(&id.id).expect("Object not found");
        let ob = ob.as_any_mut().downcast_mut::<Shape<T>>().unwrap();
        f(ob);
    }

    pub fn id_to_qbezier<T>(&self, id: &Id<T>) -> &QBezierPath {
        self.objects
            .get(&id.id)
            .expect("Object not found")
            .qbezier()
    }
}

// pub struct MeshRenderer {
//     pipeline: RenderPipeline,
//     // ...other fields
// }

pub struct QBezierRenderer {
    // TODO: remove pub
    compute_pipeline: ComputePipeline,
    stencil_pipeline: RenderPipeline,
    pub render_pipeline: RenderPipeline,
}

impl QBezierRenderer {
    const VERTEX_SIZE: usize = 32;

    pub fn new(ctx: &SurfaceContext<'_>, camera_layout: &wgpu::BindGroupLayout) -> Self {
        let compute_pipeline = Self::make_qbezier_compute_pipeline(ctx);

        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
        let vertex_layout = &[wgpu::VertexBufferLayout {
            array_stride: Self::VERTEX_SIZE as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
        }];

        let render_layout = BindGroupBuilder::new("QBezier Render Uniform Bind Group layout")
            .add_uniform_buffer(wgpu::ShaderStages::VERTEX, None)
            .build(ctx);

        let stencil_pipeline = PipelineBuilder::for_render("Stencil Pipeline", &shader)
            .vertex(vertex_layout)
            .fragment("stencil", &[])
            .depth_stencil(
                false,
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Invert,
                    pass_op: wgpu::StencilOperation::Invert,
                },
                1,
                1,
            )
            .add_bind_group_layout(camera_layout)
            .add_bind_group_layout(&render_layout)
            .build(ctx);

        let render_pipeline = PipelineBuilder::for_render("Render Pipeline", &shader)
            .vertex(vertex_layout)
            .fragment(
                "fs_main",
                &[Some(wgpu::ColorTargetState {
                    format: ctx.config.view_formats[0],
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            )
            .depth_stencil(
                true,
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                1,
                1,
            )
            .add_bind_group_layout(camera_layout)
            .add_bind_group_layout(&render_layout)
            .build(ctx);

        Self {
            compute_pipeline,
            stencil_pipeline,
            render_pipeline,
        }
    }

    pub fn render(
        &self,
        ctx: &SurfaceContext<'_>,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        cam_bind_group: &wgpu::BindGroup,
        encoder: &mut CommandEncoder,
        object: &mut Box<dyn Renderable>,
        clear: bool,
    ) {
        let qbezier = object.qbezier_mut();
        if qbezier.update_compute_buffers(ctx, &self.compute_pipeline.get_bind_group_layout(0)) {
            self.compute_pipeline
                .begin_pass("Compute Pass")
                .add_bind_group(&qbezier.compute_object.as_ref().unwrap().bind_group)
                .pass(encoder, (qbezier.num_compute_workgroups(), 1, 1));
        }
        let to_update = object.update_uniform_buff();
        let qbezier = object.qbezier();

        if to_update {
            qbezier.update_render_buffers(ctx, &qbezier.uniforms);
        }

        let render_object = qbezier.render_object.as_ref().unwrap();

        self.stencil_pipeline
            .begin_pass("Stencil Pass")
            .add_bind_group(cam_bind_group)
            .add_bind_group(&render_object.bind_group)
            .add_vertex_buffer(&render_object.vertex_buffer)
            .add_index_buffer(&render_object.index_buffer)
            .pass(
                encoder,
                &[],
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: None,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
            );

        self.render_pipeline
            .begin_pass("Render Pass")
            .add_bind_group(cam_bind_group)
            .add_bind_group(&render_object.bind_group)
            .add_vertex_buffer(&render_object.vertex_buffer)
            .add_index_buffer(&render_object.index_buffer)
            .set_stencil_reference(1)
            .pass(
                encoder,
                &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if clear {
                            wgpu::LoadOp::Clear(wgpu::Color::BLACK)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            );
    }
}

impl QBezierRenderer {
    fn make_qbezier_compute_pipeline(ctx: &impl AnyContext) -> ComputePipeline {
        let shader = ctx
            .device()
            .create_shader_module(wgpu::include_wgsl!("../compute.wgsl"));

        let layout = BindGroupBuilder::new("Compute bind group layout")
            .add_storage_buffer(ShaderStages::COMPUTE, true, None)
            .add_storage_buffer(ShaderStages::COMPUTE, false, None)
            .add_storage_buffer(ShaderStages::COMPUTE, false, None)
            .build(ctx);

        let cpipeline = PipelineBuilder::for_compute("Compute Pipeline", &shader)
            .add_bind_group_layout(&layout)
            .build(ctx);
        cpipeline
    }
}
