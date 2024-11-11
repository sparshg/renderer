use wgpu::{
    core::pipeline, ColorTargetState, DepthStencilState, FragmentState, StencilFaceState,
    VertexBufferLayout,
};

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
    label: &'a str,
    shader: &'a wgpu::ShaderModule,
    vertex: Option<&'a [VertexBufferLayout<'a>]>,
    fragment: Option<FragmentState<'a>>,
    depth_stencil: Option<DepthStencilState>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    marker: std::marker::PhantomData<T>,
}

impl<'a, T: PipelineType> PipelineBuilder<'a, T> {
    fn new<'b: 'a>(
        label: &'b str,
        shader: &'b wgpu::ShaderModule,
        vertex_buffers: Option<&'b [VertexBufferLayout<'b>]>,
    ) -> Self {
        Self {
            label,
            shader,
            fragment: None,
            vertex: vertex_buffers,
            depth_stencil: None,
            bind_group_layouts: Vec::new(),
            marker: std::marker::PhantomData,
        }
    }

    fn pipeline_layout(&self, ctx: &impl AnyContext) -> wgpu::PipelineLayout {
        ctx.device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &self.bind_group_layouts,
                push_constant_ranges: &[],
            })
    }

    pub fn add_bind_group_layout<'b: 'a>(
        &mut self,
        bind_group_layout: &'b wgpu::BindGroupLayout,
    ) -> &mut Self {
        self.bind_group_layouts.push(bind_group_layout);
        self
    }
}

impl<'a> PipelineBuilder<'a, Compute> {
    pub fn for_compute<'b: 'a>(label: &'b str, shader: &'b wgpu::ShaderModule) -> Self {
        Self::new(label, shader, None)
    }

    pub fn build(self, ctx: &impl AnyContext) -> Pipeline {
        let pipeline = ctx
            .device()
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(self.label),
                layout: Some(&self.pipeline_layout(ctx)),
                module: self.shader,
                entry_point: "main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        Pipeline {
            label: self.label.to_string(),
            pipeline: GenericPipeline::Compute(pipeline),
        }
    }
}

impl<'a> PipelineBuilder<'a, Render> {
    pub fn for_render<'b: 'a>(
        label: &'b str,
        shader: &'b wgpu::ShaderModule,
        vertex_buffers: &'b [VertexBufferLayout<'b>],
    ) -> Self {
        Self::new(label, shader, Some(vertex_buffers))
    }

    pub fn fragment<'b: 'a>(
        &mut self,
        entry_point: &'b str,
        targets: &'b [Option<ColorTargetState>],
    ) -> &mut Self {
        self.fragment = Some(FragmentState {
            module: self.shader,
            entry_point,
            targets,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });
        self
    }

    pub fn depth_stencil(
        &mut self,
        depth_write_enabled: bool,
        stencil: StencilFaceState,
        read_mask: u32,
        write_mask: u32,
    ) -> &mut Self {
        self.depth_stencil = Some(DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState {
                front: stencil,
                back: stencil,
                read_mask,
                write_mask,
            },
            bias: wgpu::DepthBiasState::default(),
        });
        self
    }

    pub fn build(self, ctx: &impl AnyContext) -> Pipeline {
        let pipeline = ctx
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(self.label),
                layout: Some(&self.pipeline_layout(ctx)),
                vertex: wgpu::VertexState {
                    module: self.shader,
                    entry_point: "vs_main",
                    buffers: self.vertex.unwrap(),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: self.fragment,
                primitive: wgpu::PrimitiveState::default(),
                multisample: wgpu::MultisampleState::default(),
                depth_stencil: self.depth_stencil,
                multiview: None,
                cache: None,
            });
        Pipeline {
            label: self.label.to_string(),
            pipeline: GenericPipeline::Render(pipeline),
        }
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
