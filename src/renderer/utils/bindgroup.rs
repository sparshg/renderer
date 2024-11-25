use std::num::NonZero;

use wgpu::BufferSize;

use super::context::AnyContext;

pub struct BindGroupBuilder {
    label: Option<String>,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupBuilder {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            entries: Vec::new(),
        }
    }

    pub fn add_storage_buffer(
        mut self,
        visibility: wgpu::ShaderStages,
        read_only: bool,
        min_binding_size: Option<NonZero<u64>>,
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size,
            },
            count: None,
        });
        self
    }

    pub fn add_uniform_buffer(
        mut self,
        visibility: wgpu::ShaderStages,
        min_binding_size: Option<BufferSize>,
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size,
            },
            count: None,
        });
        self
    }

    pub fn add_sampler(
        mut self,
        visibility: wgpu::ShaderStages,
        sampler_binding_type: wgpu::SamplerBindingType,
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty: wgpu::BindingType::Sampler(sampler_binding_type),
            count: None,
        });
        self
    }

    pub fn add_texture(
        mut self,
        visibility: wgpu::ShaderStages,
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension,
                multisampled,
            },
            count: None,
        });
        self
    }

    pub fn add_sampler_filterable(self, visibility: wgpu::ShaderStages) -> Self {
        self.add_sampler(visibility, wgpu::SamplerBindingType::Filtering)
    }

    pub fn add_texture_float_filterable_d2(
        self,
        visibility: wgpu::ShaderStages,
        multisampled: bool,
    ) -> Self {
        self.add_texture(
            visibility,
            wgpu::TextureSampleType::Float { filterable: true },
            wgpu::TextureViewDimension::D2,
            multisampled,
        )
    }

    pub fn build(self, ctx: &impl AnyContext) -> wgpu::BindGroupLayout {
        ctx.device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: self.label.as_deref(),
                entries: &self.entries,
            })
    }
}

pub trait Attach {
    fn attach(
        &self,
        ctx: &impl AnyContext,
        label: impl Into<String>,
        entries: Vec<wgpu::BindingResource<'_>>,
    ) -> wgpu::BindGroup;
}

impl Attach for wgpu::BindGroupLayout {
    fn attach(
        &self,
        ctx: &impl AnyContext,
        label: impl Into<String>,
        entries: Vec<wgpu::BindingResource<'_>>,
    ) -> wgpu::BindGroup {
        ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&label.into()),
            layout: self,
            entries: entries
                .into_iter()
                .enumerate()
                .map(|(i, resource)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource,
                })
                .collect::<Vec<_>>()
                .as_slice(),
        })
    }
}
