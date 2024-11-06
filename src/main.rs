mod camera;
mod compute;
mod texture;

use bytemuck::{Pod, Zeroable};
use camera::{Camera, CameraUniform};
use cgmath::{Vector2, Vector3};
use encase::{ArrayLength, ShaderType, StorageBuffer};
// use compute::Vertex;
use texture::Texture;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

// #[derive(Copy, Clone, Debug, ShaderType)]
// pub struct Vertex {
//     position: Vector3<f32>,
//     uv: Vector2<f32>,
// }

// impl Vertex {
//     const ATTRIBS: [wgpu::VertexAttribute; 2] =
//         wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

//     pub fn desc() -> wgpu::VertexBufferLayout<'static> {
//         wgpu::VertexBufferLayout {
//             array_stride: 24,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &Self::ATTRIBS,
//         }
//     }
// }

// impl Points {
//     fn as_wgsl_bytes(&self) -> Vec<u8> {
//         let mut buffer = encase::StorageBuffer::new(Vec::new());
//         buffer.write(self).unwrap();

//         buffer.into_inner()
//     }
// }

#[rustfmt::skip]
const VERTICES: [Vector3<f32>; 5] = [
    // Vector3 { x: 0., y: 0., z: 0.},    
    // Vector3 { x: 0.5 , y: 0., z: 0.},
    // Vector3 { x: 1., y: 1., z: 0.},
    Vector3 { x: 1., y: 0., z: 0.},    
    Vector3 { x: 1. , y: 0.41421357, z: 0.},
    Vector3 { x: 0.70710677, y: 0.70710677, z: 0.},
    Vector3 { x: 0.41421357, y: 1., z: 0.},
    Vector3 { x: 0., y: 1., z: 0.},
    // Vertex{ position: Vector3 { x: 1., y: 1.5, z: 0. }, uv: Vector2 { x: 14., y: 15. } },
    // Vertex{ position: Vector3 { x: 0., y: 2., z: 0. }, uv: Vector2 { x: 14., y: 15. } },
    // Vertex{ position: Vector3 { x: 16., y: 17., z: 18. }, uv: Vector2 { x: 19., y: 20. } },
    // Vertex{ position: Vector3 { x: 21., y: 22., z: 23. }, uv: Vector2 { x: 24., y: 25. } },
    // Vertex { position: Vector3 { x: 0.07208125, y: 0.05260625, z: 0.}, uv: Vector2 {x: 0., y: 0.} },
    // Vertex { position: Vector3 { x: 0.07208125, y: 0.02122288, z: -0.}, uv: Vector2 {x: 0.5, y: 0.} },
    // Vertex { position: Vector3 { x: 0.06163125, y: -0.00225625, z: 0.}, uv: Vector2 {x: 1., y: -1.} },
    // Vertex { position: Vector3 { x: 0.06163125, y: -0.00225625, z: 0.}, uv: Vector2 {x: 0., y: 0.} },
    // Vertex { position: Vector3 { x: 0.05225625, y: -0.02303269, z: 0.}, uv: Vector2 {x: 0.5, y: 0.} },
    // Vertex { position: Vector3 { x: 0.03604805, y: -0.03342441, z: 0.}, uv: Vector2 {x: 1., y: -1.} },
    // Vertex { position: [0.03604805, -0.03342441, 0.], uv: [0., 0.] },
    // Vertex { position: [0.01983577, -0.04381875, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.00320625, -0.04381875, 0.], uv: [1., -1.] },
    // Vertex { position: [-0.00320625, -0.04381875, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.01792568, -0.04381875, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.0296541, -0.03820781, 0.], uv: [1., -1.] },
    // Vertex { position: [-0.0296541, -0.03820781, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.04138548, -0.03259546, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.0501125, -0.021375, 0.], uv: [1., -1.] },
    // Vertex { position: [-0.0501125, -0.021375, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.06756875, 0.00106875, -0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.06756875, 0.04405625, -0.], uv: [1., -1.] },
    // Vertex { position: [-0.06756875, 0.04405625, -0.], uv: [0., 0.] },
    // Vertex { position: [-0.06756875, 0.08057819, -0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.05878125, 0.10319375, -0.], uv: [1., -1.] },
    // Vertex { position: [-0.05878125, 0.10319375, -0.], uv: [0., 0.] },
    // Vertex { position: [-0.05047654, 0.1243229, -0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.03503125, 0.1348963, -0.], uv: [1., -1.] },
    // Vertex { position: [-0.03503125, 0.1348963, -0.], uv: [0., 0.] },
    // Vertex { position: [-0.01958733, 0.14546876, -0.], uv: [0.5, 0.] },
    // Vertex { position: [0.00296875, 0.14546876, -0.], uv: [1., -1.] },
    // Vertex { position: [0.00296875, 0.14546876, -0.], uv: [0., 0.] },
    // Vertex { position: [0.01804629, 0.14546876, -0.], uv: [0.5, 0.] },
    // Vertex { position: [0.03045937, 0.13976875, -0.], uv: [1., -1.] },

    // Vertex { position: [0.03045937, 0.13976875, -0.], uv: [0., 0.] },
    // Vertex { position: [0.04287856, 0.13406597, -0.], uv: [0.5, 0.] },
    // Vertex { position: [0.05260625, 0.12266875, -0.], uv: [1., -1.] },
    // Vertex { position: [0.05260625, 0.12266875, -0.], uv: [0., 0.] },
    // Vertex { position: [0.07208125, 0.09985135, -0.], uv: [0.5, 0.] },
    // Vertex { position: [0.07208125, 0.05260625, -0.], uv: [1., -1.] },
    // Vertex { position: [0.07208125, 0.05260625, -0.], uv: [0., 0.] },
    // Vertex { position: [0.07208125, 0.05260625, -0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.00486875, 0.18323125, -0.], uv: [1., -1.] },
    //
    // Vertex { position: [-0.00486875, 0.18323125, -0.], uv: [0., 0.] },
    // Vertex { position: [-0.02695421, 0.18323125, -0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.04581152, 0.17426191, -0.], uv: [1., 1.] },
    // Vertex { position: [-0.04581152, 0.17426191, -0.], uv: [0., 0.] },
    // Vertex { position: [-0.06466535, 0.16529426, -0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.080275, 0.14736874, -0.], uv: [1., 1.] },
    // Vertex { position: [-0.080275, 0.14736874, -0.], uv: [0., 0.] },
    // Vertex { position: [-0.11150625, 0.11150405, -0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.11150625, 0.04761875, -0.], uv: [1., 1.] },
    // Vertex { position: [-0.11150625, 0.04761875, -0.], uv: [0., 0.] },
    // Vertex { position: [-0.11150625, 0.01377686, -0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.10375781, -0.01047598, 0.], uv: [1., 1.] },
    // Vertex { position: [-0.10375781, -0.01047598, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.09601022, -0.03472614, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.0805125, -0.0494, 0.], uv: [1., 1.] },

    // Vertex { position: [-0.0805125, -0.0494, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.04953443, -0.07873125, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.00819375, -0.07873125, 0.], uv: [1., 1.] },
    // Vertex { position: [-0.00819375, -0.07873125, 0.], uv: [0., 0.] },
    // Vertex { position: [0.02148037, -0.07873125, 0.], uv: [0.5, 0.] },
    // Vertex { position: [0.03954375, -0.06923125, 0.], uv: [1., 1.] },
    // Vertex { position: [0.03954375, -0.06923125, 0.], uv: [0., 0.] },
    // Vertex { position: [0.05757167, -0.05974989, 0.], uv: [0.5, 0.] },
    // Vertex { position: [0.07041875, -0.04025625, 0.], uv: [1., 1.] },
    // Vertex { position: [0.07041875, -0.04025625, 0.], uv: [0., 0.] },
    // Vertex { position: [0.07113131, -0.09276237, 0.], uv: [0.5, 0.] },
    // Vertex { position: [0.06210625, -0.11316875, 0.], uv: [1., -1.] },
    // Vertex { position: [0.06210625, -0.11316875, 0.], uv: [0., 0.] },
    // Vertex { position: [0.05451531, -0.13050993, 0.], uv: [0.5, 0.] },
    // Vertex { position: [0.03841934, -0.13917871, 0.], uv: [1., -1.] },
    // Vertex { position: [0.03841934, -0.13917871, 0.], uv: [0., 0.] },
    // Vertex { position: [0.0223303, -0.14784375, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.00225625, -0.14784375, 0.], uv: [1., -1.] },
    // Vertex { position: [-0.00225625, -0.14784375, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.03338359, -0.14784375, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.04785625, -0.13359375, 0.], uv: [1., -1.] },
    // Vertex { position: [-0.04785625, -0.13359375, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.05713685, -0.12431316, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.06020625, -0.10723125, 0.], uv: [1., -1.] },
    // Vertex { position: [-0.06020625, -0.10723125, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.0819375, -0.10723125, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.10366875, -0.10723125, 0.], uv: [1., -1.] },
    // Vertex { position: [-0.10366875, -0.10723125, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.10199864, -0.12682721, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.09416133, -0.14122714, 0.], uv: [1., 1.] },
    // Vertex { position: [-0.09416133, -0.14122714, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.08632226, -0.15563034, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.07231875, -0.164825, 0.], uv: [1., 1.] },
    // Vertex { position: [-0.07231875, -0.164825, 0.], uv: [0., 0.] },
    // Vertex { position: [-0.04428599, -0.18323125, 0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.00320625, -0.18323125, 0.], uv: [1., 1.] },
    // Vertex { position: [-0.00320625, -0.18323125, 0.], uv: [0., 0.] },
    // Vertex { position: [0.03373603, -0.18323125, 0.], uv: [0.5, 0.] },
    // Vertex { position: [0.05884805, -0.17029122, 0.], uv: [1., 1.] },
    // Vertex { position: [0.05884805, -0.17029122, 0.], uv: [0., 0.] },
    // Vertex { position: [0.08396496, -0.15734865, 0.], uv: [0.5, 0.] },
    // Vertex { position: [0.09725625, -0.13145626, 0.], uv: [1., 1.] },
    // Vertex { position: [0.09725625, -0.13145626, 0.], uv: [0., 0.] },
    // Vertex { position: [0.11150625, -0.1034461, 0.], uv: [0.5, 0.] },
    // Vertex { position: [0.11150625, -0.05498125, 0.], uv: [1., 1.] },
    // Vertex { position: [0.11150625, -0.05498125, 0.], uv: [0., 0.] },
    // Vertex { position: [0.11150625, 0.06068125, -0.], uv: [0.5, 0.] },
    // Vertex { position: [0.11150625, 0.17634375, -0.], uv: [1., 1.] },
    // Vertex { position: [0.11150625, 0.17634375, -0.], uv: [0., 0.] },
    // Vertex { position: [0.09179375, 0.17634375, -0.], uv: [0.5, 0.] },
    // Vertex { position: [0.07208125, 0.17634375, -0.], uv: [1., 1.] },
    // Vertex { position: [0.07208125, 0.17634375, -0.], uv: [0., 0.] },
    // Vertex { position: [0.07208125, 0.1603125, -0.], uv: [0.5, 0.] },
    // Vertex { position: [0.07208125, 0.14428125, -0.], uv: [1., 1.] },
    // Vertex { position: [0.07208125, 0.14428125, -0.], uv: [0., 0.] },
    // Vertex { position: [0.05946112, 0.16020134, -0.], uv: [0.5, 0.] },
    // Vertex { position: [0.04738125, 0.16850625, -0.], uv: [1., 1.] },
    // Vertex { position: [0.04738125, 0.16850625, -0.], uv: [0., 0.] },
    // Vertex { position: [0.02509356, 0.18323125, -0.], uv: [0.5, 0.] },
    // Vertex { position: [-0.00486875, 0.18323125, -0.], uv: [1., 1.] },
];

// #[rustfmt::skip]
// const VERTICES: &[Vertex] = &[
//    Vertex { position: [ 0. , 0., 0.], uv: [0., 0.] },
//    Vertex { position: [ 0.5, 0., 0.], uv: [0.5, 0.] },
//    Vertex { position: [ 1. , 1., 0.], uv: [1., -1.] },

//    Vertex { position: [ -1. , 0., 0.], uv: [0., 0.] },
//    Vertex { position: [ -0.5, 0., 0.], uv: [0.5, 0.] },
//    Vertex { position: [ 0.  , 1., 0.], uv: [1., 1.] },
//    Vertex { position: [ 1.        ,  0.        , 0.], uv: [0., 0.] },
//    Vertex { position: [ 1.        ,  0.41421357, 0.], uv: [0.5, 0.] },
//    Vertex { position: [ 0.70710677,  0.70710677, 0.], uv: [1., 1.] },
//    Vertex { position: [ 0.70710677,  0.70710677, 0.], uv: [0., 0.] },
//    Vertex { position: [ 0.41421357,  1.        , 0.], uv: [0.5, 0.] },
//    Vertex { position: [ 0.        ,  1.        , 0.], uv: [1., 1.] },
//    Vertex { position: [ 0.        ,  1.        , 0.], uv: [0., 0.] },
//    Vertex { position: [-0.41421357,  1.        , 0.], uv: [0.5, 0.] },
//    Vertex { position: [-0.70710677,  0.70710677, 0.], uv: [1., 1.] },
//    Vertex { position: [-0.70710677,  0.70710677, 0.], uv: [0., 0.] },
//    Vertex { position: [-1.        ,  0.41421357, 0.], uv: [0.5, 0.] },
//    Vertex { position: [-1.        ,  0.        , 0.], uv: [1., 1.] },
//    Vertex { position: [-1.        ,  0.        , 0.], uv: [0., 0.] },
//    Vertex { position: [-1.        , -0.41421357, 0.], uv: [0.5, 0.] },
//    Vertex { position: [-0.70710677, -0.70710677, 0.], uv: [1., 1.] },
//    Vertex { position: [-0.70710677, -0.70710677, 0.], uv: [0., 0.] },
//    Vertex { position: [-0.41421357, -1.        , 0.], uv: [0.5, 0.] },
//    Vertex { position: [-0.        , -1.        , 0.], uv: [1., 1.] },
//    Vertex { position: [-0.        , -1.        , 0.], uv: [0., 0.] },
//    Vertex { position: [ 0.41421357, -1.        , 0.], uv: [0.5, 0.] },
//    Vertex { position: [ 0.70710677, -0.70710677, 0.], uv: [1., 1.] },
//    Vertex { position: [ 0.70710677, -0.70710677, 0.], uv: [0., 0.] },
//    Vertex { position: [ 1.        , -0.41421357, 0.], uv: [0.5, 0.] },
//    Vertex { position: [ 1.        ,  0.        , 0.], uv: [1., 1.] },
// ];

// const INDICES: &[u16] = &[0, 1, 2];

struct DeviceQueue {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    adapter: wgpu::Adapter,
}

impl DeviceQueue {
    async fn new(compatible_surface: Option<&wgpu::Surface<'_>>) -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: compatible_surface,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        Self {
            device,
            queue,
            adapter,
        }
    }
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    rpipeline: wgpu::RenderPipeline,
    cpipeline: compute::ComputePipeline,
    // vertex_buffer: wgpu::Buffer,
    // index_buffer: wgpu::Buffer,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    diffuse_bind_group: wgpu::BindGroup,
    depth_texture: Texture,
    // indices: Vec<u32>,
    pub window: &'a Window,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> Self {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // let DeviceQueue {
        //     device,
        //     queue,
        //     adapter,
        // } = DeviceQueue::new(Some(&surface)).await;
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("happy-tree.png"),
            "happy-tree.png",
        )
        .unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera = Camera::new();
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let rpipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 32,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multiview: None,
            cache: None,
        });

        let cpipeline = compute::ComputePipeline::new(&device);

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            (config.width, config.height),
            "depth_texture",
        );

        Self {
            surface,
            device,
            queue,
            config,
            size,
            rpipeline,
            cpipeline,
            // vertex_buffer,
            // index_buffer,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            diffuse_bind_group,
            depth_texture,
            window,
            // indices,
        }
    }

    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.cpipeline.pipeline);
            cpass.set_bind_group(0, &self.cpipeline.bind_group, &[]);
            cpass.dispatch_workgroups(((VERTICES.len() as f32) / 64.0).ceil() as u32, 1, 1);
        }
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.rpipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.cpipeline.vert_buff.slice(..));
            rpass.set_index_buffer(self.cpipeline.ind_buff.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..self.cpipeline.ind_buff.size() as u32 / 4, 0, 0..1);
        }
        // let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
        //     label: None,
        //     size: self.cpipeline.vert_buff.size(),
        //     usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });
        // encoder.copy_buffer_to_buffer(
        //     &self.cpipeline.vert_buff,
        //     0,
        //     &staging_buffer,
        //     0,
        //     self.cpipeline.vert_buff.size(),
        // );
        self.queue.submit(Some(encoder.finish()));
        frame.present();

        // let staging_buffer = staging_buffer.slice(..);
        // staging_buffer.map_async(wgpu::MapMode::Read, |_| {});
        // self.device.poll(wgpu::Maintain::Wait);
        // let v = staging_buffer.get_mapped_range();
        // let buffer = StorageBuffer::new(v.as_ref());
        // let mut p: Vec<Vertex> = Vec::new();
        // buffer.read(&mut p).unwrap();

        // // let v: Vec<Vertex> = bytemuck::cast_slice(&v).to_vec();
        // println!("{:#?}", p);
        // // staging_buffer.;
        // panic!("");

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        self.depth_texture = texture::Texture::create_depth_texture(
            &self.device,
            (self.config.width, self.config.height),
            "depth_texture",
        );

        self.update();
        // On macos the window needs to be redrawn manually after resizing
        self.window.request_redraw();
    }

    fn input(&mut self, event: &WindowEvent) {
        self.camera.process_inputs(event)
    }

    fn update(&mut self) {
        self.camera.update_camera();
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut state = State::new(&window).await;
    event_loop
        .run(move |event, target| {
            let Event::WindowEvent { event, .. } = event else {
                return;
            };
            state.input(&event);

            match event {
                WindowEvent::Resized(new_size) => state.resize(new_size),
                WindowEvent::RedrawRequested => {
                    state.window.request_redraw();
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.size);
                        }

                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("OutOfMemory");
                            target.exit();
                        }

                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }
                WindowEvent::CloseRequested => target.exit(),
                _ => {}
            };
        })
        .unwrap();
}

pub fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    env_logger::init();
    pollster::block_on(run(event_loop, window));
}
