// use context::SurfaceWrapper;

mod bindgroup;
mod context;
mod pipeline;
mod window;

pub use bindgroup::{Attach, BindGroupBuilder};
pub use context::AnyContext;
pub use context::Context;
pub use context::SurfaceContext;
pub use pipeline::{Pipeline, PipelineBuilder};
use wgpu::CommandEncoder;
use wgpu::ComputePipeline;
use wgpu::RenderPipeline;
use wgpu::ShaderStages;
pub use window::App;
pub use window::Window;

use crate::camera::Camera;
use crate::object::QBezier;
use crate::texture::Texture;

async fn test() {
    let win = window::Window::new("test");
    let w = win.get_window();
    let ctx = context::Context::init().await.attach_window(&w);

    let shader = ctx
        .device
        .create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
    let targets: [Option<wgpu::ColorTargetState>; 0] = [];
}

pub trait Renderable {
    const VERTEX_SIZE: usize;
    fn update_compute_buffers(
        &mut self,
        ctx: &impl AnyContext,
        layout: &wgpu::BindGroupLayout,
    ) -> bool;
    fn update_render_buffers(&mut self, ctx: &impl AnyContext, layout: &wgpu::BindGroupLayout);
    fn get_compute_bgroup(&self) -> &wgpu::BindGroup;
    fn num_compute_workgroups(&self) -> u32;
    fn get_vertex_buffer(&self) -> &wgpu::Buffer;
    fn get_index_buffer(&self) -> &wgpu::Buffer;
    fn get_render_bgroup(&self) -> &wgpu::BindGroup;
}

pub struct Renderer {
    pub camera: Camera,
    pub depth_texture: Texture,
    qbezier_compute_pipeline: Pipeline<ComputePipeline>,
    qbezier_render_pipelines: Vec<Pipeline<RenderPipeline>>,
    qbezier_compute_layout: wgpu::BindGroupLayout,
    qbezier_render_layout: wgpu::BindGroupLayout,
}

impl Renderer {
    pub async fn new(ctx: &SurfaceContext<'_>) -> Self {
        let (qbezier_compute_pipeline, qbezier_compute_layout) =
            Self::make_qbezier_compute_pipeline(ctx);

        let camera = Camera::new(ctx);
        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
        let vertex_layout = &[wgpu::VertexBufferLayout {
            array_stride: <QBezier as Renderable>::VERTEX_SIZE as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
        }];

        let qbezier_render_layout =
            BindGroupBuilder::new("QBezier Render Uniform Bind Group layout")
                .add_uniform_buffer(wgpu::ShaderStages::VERTEX, None)
                .build(ctx);

        let spipeline = PipelineBuilder::for_render("Stencil Pipeline", &shader)
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
            .add_bind_group_layout(&camera.bind_group_layout)
            .add_bind_group_layout(&qbezier_render_layout)
            .build(ctx);

        let rpipeline = PipelineBuilder::for_render("Render Pipeline", &shader)
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
            .add_bind_group_layout(&camera.bind_group_layout)
            .add_bind_group_layout(&qbezier_render_layout)
            .build(ctx);

        let depth_texture = Texture::create_depth_texture(
            &ctx.device,
            (ctx.config.width, ctx.config.height),
            "Depth Texture",
        );

        Self {
            camera,
            depth_texture,
            qbezier_render_pipelines: vec![spipeline, rpipeline],
            qbezier_compute_pipeline,
            qbezier_compute_layout,
            qbezier_render_layout,
        }
    }

    pub fn render_qbezier(
        &self,
        ctx: &SurfaceContext<'_>,
        view: &wgpu::TextureView,
        encoder: &mut CommandEncoder,
        qbezier: &mut QBezier,
    ) {
        if qbezier.update_compute_buffers(ctx, &self.qbezier_compute_layout) {
            self.qbezier_compute_pipeline
                .begin_pass("Compute Pass")
                .add_bind_group(qbezier.get_compute_bgroup())
                .pass(encoder, (qbezier.num_compute_workgroups(), 1, 1));
        }

        qbezier.update_render_buffers(ctx, &self.qbezier_render_layout);

        self.qbezier_render_pipelines[0]
            .begin_pass("Stencil Pass")
            .add_bind_group(&self.camera.bind_group)
            .add_bind_group(qbezier.get_render_bgroup())
            .add_vertex_buffer(qbezier.get_vertex_buffer())
            .add_index_buffer(qbezier.get_index_buffer())
            .pass(
                encoder,
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

        self.qbezier_render_pipelines[1]
            .begin_pass("Render Pass")
            .add_bind_group(&self.camera.bind_group)
            .add_bind_group(qbezier.get_render_bgroup())
            .add_vertex_buffer(qbezier.get_vertex_buffer())
            .add_index_buffer(qbezier.get_index_buffer())
            .set_stencil_reference(1)
            .pass(
                encoder,
                &[Some(wgpu::RenderPassColorAttachment {
                    view,
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

impl Renderer {
    fn make_qbezier_compute_pipeline(
        ctx: &impl AnyContext,
    ) -> (Pipeline<ComputePipeline>, wgpu::BindGroupLayout) {
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
        (cpipeline, layout)
    }
}
