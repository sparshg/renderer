use wgpu::{CommandEncoder, ComputePipeline, RenderPipeline, ShaderStages};

use super::{
    utils::pipeline::IntoPass, AnyContext, BindGroupBuilder, PipelineBuilder, Renderable,
    SurfaceContext,
};

pub struct QBezierRenderer {
    // TODO: remove pub
    compute_pipeline: ComputePipeline,
    stencil_pipeline: RenderPipeline,
    pub render_pipeline: RenderPipeline,
}

impl QBezierRenderer {
    // TODO: This is in shape as well
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

    pub fn compute_layout(&self) -> wgpu::BindGroupLayout {
        self.compute_pipeline.get_bind_group_layout(0)
    }

    pub fn render_layout(&self) -> wgpu::BindGroupLayout {
        self.render_pipeline.get_bind_group_layout(1)
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
        if object.update_compute_buffers(ctx, &self.compute_layout()) {
            self.compute_pipeline
                .begin_pass("Compute Pass")
                .add_bind_group(&object.get_compute_object().bind_group)
                .pass(encoder, (object.num_compute_workgroups(), 1, 1));
        }
        object.update_render_buffers(ctx);

        let render_object = object.get_render_object();

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
