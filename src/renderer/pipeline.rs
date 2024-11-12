use wgpu::{
    ColorTargetState, ComputePipeline, DepthStencilState, FragmentState, RenderPipeline, StencilFaceState, VertexBufferLayout,
};

use super::context::AnyContext;

trait PipelineType {}
pub struct RenderNoVertex;
pub struct Render;
pub struct Compute;
impl PipelineType for Render {}
impl PipelineType for RenderNoVertex {}
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
    fn new<'b: 'a>(label: &'b str, shader: &'b wgpu::ShaderModule) -> Self {
        Self {
            label,
            shader,
            fragment: None,
            vertex: None,
            depth_stencil: None,
            bind_group_layouts: Vec::new(),
            marker: std::marker::PhantomData,
        }
    }

    fn pipeline_layout(&self, ctx: &impl AnyContext) -> wgpu::PipelineLayout {
        let label = self.label.to_string() + " layout";
        ctx.device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&label),
                bind_group_layouts: &self.bind_group_layouts,
                push_constant_ranges: &[],
            })
    }

    pub fn add_bind_group_layout<'b: 'a>(
        mut self,
        bind_group_layout: &'b wgpu::BindGroupLayout,
    ) -> Self {
        self.bind_group_layouts.push(bind_group_layout);
        self
    }
}

impl<'a> PipelineBuilder<'a, Compute> {
    pub fn for_compute<'b: 'a>(label: &'b str, shader: &'b wgpu::ShaderModule) -> Self {
        Self::new(label, shader)
    }

    pub fn build(self, ctx: &'a impl AnyContext) -> Pipeline<ComputePipeline> {
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
        Pipeline::new(pipeline, self.bind_group_layouts.len(), 0)
    }
}

impl<'a> PipelineBuilder<'a, RenderNoVertex> {
    pub fn for_render<'b: 'a>(label: &'b str, shader: &'b wgpu::ShaderModule) -> Self {
        Self::new(label, shader)
    }

    pub fn vertex<'b: 'a>(
        self,
        vertex_buffers: &'b [VertexBufferLayout<'b>],
    ) -> PipelineBuilder<'a, Render> {
        PipelineBuilder {
            label: self.label,
            shader: self.shader,
            vertex: Some(vertex_buffers),
            fragment: self.fragment,
            depth_stencil: self.depth_stencil,
            bind_group_layouts: self.bind_group_layouts,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a> PipelineBuilder<'a, Render> {
    pub fn fragment<'b: 'a>(
        mut self,
        entry_point: &'b str,
        targets: &'b [Option<ColorTargetState>],
    ) -> Self {
        self.fragment = Some(FragmentState {
            module: self.shader,
            entry_point,
            targets,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });
        self
    }

    pub fn depth_stencil(
        mut self,
        depth_write_enabled: bool,
        stencil: StencilFaceState,
        read_mask: u32,
        write_mask: u32,
    ) -> Self {
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

    pub fn build(self, ctx: &'a impl AnyContext) -> Pipeline<RenderPipeline> {
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
        Pipeline::new(
            pipeline,
            self.bind_group_layouts.len(),
            self.vertex.unwrap().len(),
        )
    }
}

pub struct Pipeline<T> {
    pub pipeline: T, //TODO: no public
    num_bind_groups: usize,
    num_vertex_buffers: usize,
}

pub struct PipelinePass<'a, T> {
    label: Option<String>,
    pipeline: &'a T,
    num_bind_groups: usize,
    num_vertex_buffers: usize,
    bind_groups: Vec<&'a wgpu::BindGroup>,
    vertex_buffers: Vec<&'a wgpu::Buffer>,
    index_buffer: Option<&'a wgpu::Buffer>,
}

impl<T> Pipeline<T> {
    fn new(pipeline: T, num_bind_groups: usize, num_vertex_buffers: usize) -> Self {
        Self {
            pipeline,
            num_bind_groups,
            num_vertex_buffers,
        }
    }

    pub fn begin_pass(&self, label: impl Into<String>) -> PipelinePass<'_, T> {
        PipelinePass {
            label: Some(label.into()),
            pipeline: &self.pipeline,
            num_bind_groups: self.num_bind_groups,
            num_vertex_buffers: self.num_vertex_buffers,
            bind_groups: Vec::with_capacity(self.num_bind_groups),
            vertex_buffers: Vec::with_capacity(self.num_vertex_buffers),
            index_buffer: None,
        }
    }
}

impl<'a, T> PipelinePass<'a, T> {
    pub fn add_bind_group<'b: 'a>(mut self, bind_group: &'b wgpu::BindGroup) -> Self {
        self.bind_groups.push(bind_group);
        self
    }
}

impl<'a> PipelinePass<'a, RenderPipeline> {
    pub fn add_vertex_buffer<'b: 'a>(mut self, vertex_buffer: &'a wgpu::Buffer) -> Self {
        self.vertex_buffers.push(vertex_buffer);
        self
    }

    pub fn add_index_buffer<'b: 'a>(mut self, index_buffer: &'a wgpu::Buffer) -> Self {
        self.index_buffer = Some(index_buffer);
        self
    }

    pub fn pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_attachments: &[Option<wgpu::RenderPassColorAttachment<'_>>],
        depth_stencil_attachment: Option<wgpu::RenderPassDepthStencilAttachment<'_>>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: self.label.as_deref(),
            color_attachments,
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(self.pipeline);
        for (i, bind_group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, bind_group, &[]);
        }
        for (i, vertex_buffer) in self.vertex_buffers.iter().enumerate() {
            render_pass.set_vertex_buffer(i as u32, vertex_buffer.slice(..));
        }
        render_pass.set_index_buffer(
            self.index_buffer.unwrap().slice(..),
            wgpu::IndexFormat::Uint32,
        );
        let indices =
            self.index_buffer.unwrap().size() / std::mem::size_of::<u32>() as wgpu::BufferAddress;
        render_pass.draw_indexed(0..indices as u32, 0, 0..1);
    }
}

impl PipelinePass<'_, ComputePipeline> {
    pub fn pass(&self, encoder: &mut wgpu::CommandEncoder, dispatch: (u32, u32, u32)) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: self.label.as_deref(),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(self.pipeline);
        for (i, bind_group) in self.bind_groups.iter().enumerate() {
            compute_pass.set_bind_group(i as u32, bind_group, &[]);
        }
        compute_pass.dispatch_workgroups(dispatch.0, dispatch.1, dispatch.2);
    }
}
