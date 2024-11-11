use std::num::NonZero;

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
        min_binding_size: Option<NonZero<u64>>,
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
        sampler_binding_type: wgpu::SamplerBindingType,
        visibility: wgpu::ShaderStages,
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

    pub fn add_sampler_filterable(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        self
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
