use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use encase::ArrayLength;
use wgpu::util::DeviceExt;

use crate::VERTEX_STRUCT_SIZE;

pub const POS: &[[f32; 3]; 51] = &[
    // [0.07208125, 0.05260625, -0.],
    // [0.07208125, 0.02122288, -0.],
    // [0.06163125, -0.00225625, 0.],
    // [0.05225625, -0.02303269, 0.],
    // [0.03604805, -0.03342441, 0.],
    // [0.01983577, -0.04381875, 0.],
    // [-0.00320625, -0.04381875, 0.],
    // [-0.01792568, -0.04381875, 0.],
    // [-0.0296541, -0.03820781, 0.],
    // [-0.04138548, -0.03259546, 0.],
    // [-0.0501125, -0.021375, 0.],
    // [-0.06756875, 0.00106875, -0.],
    // [-0.06756875, 0.04405625, -0.],
    // [-0.06756875, 0.08057819, -0.],
    // [-0.05878125, 0.10319375, -0.],
    // [-0.05047654, 0.1243229, -0.],
    // [-0.03503125, 0.1348963, -0.],
    // [-0.01958733, 0.14546876, -0.],
    // [0.00296875, 0.14546876, -0.],
    // [0.01804629, 0.14546876, -0.],
    // [0.03045937, 0.13976875, -0.],
    // [0.04287856, 0.13406597, -0.],
    // [0.05260625, 0.12266875, -0.],
    // [0.07208125, 0.09985135, -0.],
    // [0.07208125, 0.05260625, -0.],
    //
    // [0.07208125, 0.05260625, -0.],
    [-0.00486875, 0.18323125, -0.],
    [-0.02695421, 0.18323125, -0.],
    [-0.04581152, 0.17426191, -0.],
    [-0.06466535, 0.16529426, -0.],
    [-0.080275, 0.14736874, -0.],
    [-0.11150625, 0.11150405, -0.],
    [-0.11150625, 0.04761875, -0.],
    [-0.11150625, 0.01377686, -0.],
    [-0.10375781, -0.01047598, 0.],
    [-0.09601022, -0.03472614, 0.],
    [-0.0805125, -0.0494, 0.],
    [-0.04953443, -0.07873125, 0.],
    [-0.00819375, -0.07873125, 0.],
    [0.02148037, -0.07873125, 0.],
    [0.03954375, -0.06923125, 0.],
    [0.05757167, -0.05974989, 0.],
    [0.07041875, -0.04025625, 0.],
    [0.07113131, -0.09276237, 0.],
    [0.06210625, -0.11316875, 0.],
    [0.05451531, -0.13050993, 0.],
    [0.03841934, -0.13917871, 0.],
    [0.0223303, -0.14784375, 0.],
    [-0.00225625, -0.14784375, 0.],
    [-0.03338359, -0.14784375, 0.],
    [-0.04785625, -0.13359375, 0.],
    [-0.05713685, -0.12431316, 0.],
    [-0.06020625, -0.10723125, 0.],
    [-0.0819375, -0.10723125, 0.],
    [-0.10366875, -0.10723125, 0.],
    [-0.10199864, -0.12682721, 0.],
    [-0.09416133, -0.14122714, 0.],
    [-0.08632226, -0.15563034, 0.],
    [-0.07231875, -0.164825, 0.],
    [-0.04428599, -0.18323125, 0.],
    [-0.00320625, -0.18323125, 0.],
    [0.03373603, -0.18323125, 0.],
    [0.05884805, -0.17029122, 0.],
    [0.08396496, -0.15734865, 0.],
    [0.09725625, -0.13145626, 0.],
    [0.11150625, -0.1034461, 0.],
    [0.11150625, -0.05498125, 0.],
    [0.11150625, 0.06068125, -0.],
    [0.11150625, 0.17634375, -0.],
    [0.09179375, 0.17634375, -0.],
    [0.07208125, 0.17634375, -0.],
    [0.07208125, 0.1603125, -0.],
    [0.07208125, 0.14428125, -0.],
    [0.05946112, 0.16020134, -0.],
    [0.04738125, 0.16850625, -0.],
    [0.02509356, 0.18323125, -0.],
    [-0.00486875, 0.18323125, -0.],
];
pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub point_buff: wgpu::Buffer,
    pub vert_buff: wgpu::Buffer,
    pub ind_buff: wgpu::Buffer,
}

impl ComputePipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let mut buffer = encase::StorageBuffer::new(Vec::new());
        buffer.write(&POS.map(|x| Vector3::from(x))).unwrap();
        let buffer = buffer.into_inner();
        let point_buff = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point Buffer"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: &buffer,
        });
        let vert_buff = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (POS.len() as u64 / 2 + 1) * 3 * VERTEX_STRUCT_SIZE as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ind_buff = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (POS.len() / 2 * 6 * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: point_buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: vert_buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: ind_buff.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            point_buff,
            vert_buff,
            ind_buff,
        }
    }
}
