// use context::SurfaceWrapper;

mod bindgroup;
mod context;
mod pipeline;
mod window;

pub use bindgroup::{Attach, BindGroupBuilder};
pub use context::AnyContext;
pub use context::Context;
pub use context::SurfaceContext;
pub use pipeline::{Pipeline, PipelineBuilder, PipelinePass};
use wgpu::ComputePipeline;
use wgpu::RenderPipeline;
use wgpu::ShaderStages;
pub use window::App;
pub use window::Window;

use crate::camera::Camera;
use crate::object::QBezier;

async fn test() {
    let win = window::Window::new("test");
    let w = win.get_window();
    let ctx = context::Context::init().await.attach_window(&w);

    // // let SurfaceWrapper { surface, config } = window::Window::init_surface(&ctx, &win.window);
    // // ctx.surface = Some(surface);
    // // ctx.config = Some(config);
    let shader = ctx
        .device
        .create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
    let targets: [Option<wgpu::ColorTargetState>; 0] = [];
    // let rp = pipeline::RenderPipelineBuilder::for_render(Some("label"), &module);
    // let rpipeline = PipelineBuilder::for_render("Render Pipeline", &shader)
    //     .vertex(&[wgpu::VertexBufferLayout {
    //         array_stride: 32,
    //         step_mode: wgpu::VertexStepMode::Vertex,
    //         attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
    //     }])
    //     .fragment("fs_main")
    //     .depth_stencil(true, stencil_r, 1, 1)
    //     .build(&ctx);
}

pub trait Renderable {
    const VERTEX_SIZE: usize;
    fn update_buffers(&mut self, ctx: &impl AnyContext);
    fn compute(&self, pipeline: &Pipeline<ComputePipeline>, encoder: &mut wgpu::CommandEncoder);
    fn render(&self, pipeline: &[Pipeline<RenderPipeline>], encoder: &mut wgpu::CommandEncoder);
}

pub struct Renderer {
    camera: Camera,
    qbezier_render_pipelines: Vec<Pipeline<RenderPipeline>>,
    qbezier_compute_pipeline: Pipeline<ComputePipeline>,
    pub qbezier_compute_layout: wgpu::BindGroupLayout,
}

impl Renderer {
    async fn new(ctx: &SurfaceContext<'_>) -> Self {
        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("../compute.wgsl"));

        let compute_bglayout = BindGroupBuilder::new("Compute bind group layout")
            .add_storage_buffer(ShaderStages::COMPUTE, true, None)
            .add_storage_buffer(ShaderStages::COMPUTE, false, None)
            .add_storage_buffer(ShaderStages::COMPUTE, false, None)
            .build(ctx);

        let cpipeline = PipelineBuilder::for_compute("Compute Pipeline", &shader)
            .add_bind_group_layout(&compute_bglayout)
            .build(ctx);

        let camera = Camera::new(ctx);

        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
        let vertex_layout = &[wgpu::VertexBufferLayout {
            array_stride: <QBezier as Renderable>::VERTEX_SIZE as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
        }];
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
            .build(ctx);
        Self {
            camera: Camera::new(ctx),
            qbezier_render_pipelines: vec![spipeline, rpipeline],
            qbezier_compute_pipeline: cpipeline,
            qbezier_compute_layout: compute_bglayout,
        }
    }

    pub fn render_qbezier(&self, ctx: SurfaceContext<'_>, qbezier: &QBezier) {
        let frame = ctx.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        qbezier.compute(&self.qbezier_compute_pipeline, &mut encoder);
        // self.qbezier_compute_pipeline
        //     .begin_pass("Compute Pass")
        //     .add_bind_group(&qbezier.compute_bgroup.as_ref().unwrap())
        //     .pass(
        //         &mut encoder,
        //         (
        //             (((qbezier.points.len() / 2) as f32) / 64.0).ceil() as u32,
        //             1,
        //             1,
        //         ),
        //     );
    }
}
