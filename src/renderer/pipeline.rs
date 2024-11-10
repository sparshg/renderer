use wgpu::core::pipeline;

use super::{
    context::{AnyContext, Context},
    SurfaceContext,
};

trait PipelineType {}
struct Render;
struct Compute;
impl PipelineType for Render {}
impl PipelineType for Compute {}

pub struct PipelineBuilder<'a, T: PipelineType> {
    label: String,
    shader: &'a wgpu::ShaderModule,
    fragment: bool,
    // bind_group: wgpu::BindGroup,
    marker: std::marker::PhantomData<T>,
}

impl<'a> PipelineBuilder<'a, Compute> {
    pub fn for_compute<'b: 'a>(label: impl Into<String>, shader: &'b wgpu::ShaderModule) -> Self {
        Self {
            label: label.into(),
            shader,
            fragment: false,
            marker: std::marker::PhantomData,
        }
    }
    pub fn build(&self, ctx: &impl AnyContext) {
        let pipeline_layout =
            ctx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let pipeline = ctx
            .device()
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: self.shader,
                entry_point: "main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
    }
}

impl<'a> PipelineBuilder<'a, Render> {
    pub fn for_render<'b: 'a>(label: impl Into<String>, shader: &'b wgpu::ShaderModule) -> Self {
        Self {
            label: label.into(),
            shader,
            fragment: true,
            marker: std::marker::PhantomData,
        }
    }

    pub fn disable_fragment(&mut self) -> &mut Self {
        self.fragment = false;
        self
    }

    // pub fn enable_depth(compare: Compa)

    pub fn build(&mut self, ctx: &SurfaceContext) -> &mut Self {
        //     let bind_group_layout =
        //         ctx.device
        //             .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //                 label: None,
        //                 entries: &[wgpu::BindGroupLayoutEntry {
        //                     binding: 0,
        //                     visibility: wgpu::ShaderStages::FRAGMENT,
        //                     ty: wgpu::BindingType::Buffer {
        //                         ty: wgpu::BufferBindingType::Uniform,
        //                         has_dynamic_offset: false,
        //                         min_binding_size: None,
        //                     },
        //                     count: None,
        //                 }],
        //             });

        //     let pipeline_layout = ctx
        //         .device
        //         .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //             label: None,
        //             bind_group_layouts: &[&bind_group_layout],
        //             push_constant_ranges: &[],
        //         });

        //     let targets = &[Some(wgpu::ColorTargetState {
        //         format: ctx.config.view_formats[0],
        //         blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        //         write_mask: wgpu::ColorWrites::ALL,
        //     })];

        //     let pipeline = ctx
        //         .device
        //         .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //             label: self.label.as_deref(),
        //             layout: Some(&pipeline_layout),
        //             vertex: wgpu::VertexState {
        //                 module: self.shader,
        //                 entry_point: "vs_main",
        //                 buffers: &[wgpu::VertexBufferLayout {
        //                     array_stride: VERTEX_STRUCT_SIZE,
        //                     step_mode: wgpu::VertexStepMode::Vertex,
        //                     attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
        //                 }],
        //                 compilation_options: wgpu::PipelineCompilationOptions::default(),
        //             },
        //             fragment: self.fragment.then(|| wgpu::FragmentState {
        //                 module: self.shader,
        //                 entry_point: "fs_main",
        //                 targets,
        //                 compilation_options: wgpu::PipelineCompilationOptions::default(),
        //             }),
        //             primitive: wgpu::PrimitiveState::default(),
        //             multisample: wgpu::MultisampleState::default(),
        //             depth_stencil: None,
        //             multiview: None,
        //             cache: None,
        //         });
        self
    }

    // pub fn
}

enum GenericPipeline {
    Render(wgpu::RenderPipeline),
    Compute(wgpu::ComputePipeline),
}

struct Pipeline {
    label: String,
    pipeline: GenericPipeline,
    bind_group: wgpu::BindGroup,
}

impl Pipeline {
    pub fn pass(&self, encoder: &mut wgpu::CommandEncoder) {
        match &self.pipeline {
            GenericPipeline::Render(pipeline) => {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                render_pass.set_pipeline(pipeline);
            }
            GenericPipeline::Compute(pipeline) => {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(pipeline);
            }
        }
    }
}
