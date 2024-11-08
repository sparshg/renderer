mod camera;
mod compute;
mod renderer;
mod texture;
use camera::{Camera, CameraUniform};
use compute::POS;
use encase::ShaderType;
// use compute::Vertex;
use texture::Texture;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

pub const VERTEX_STRUCT_SIZE: u64 = 32;

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    rpipeline: wgpu::RenderPipeline,
    spipeline: wgpu::RenderPipeline,
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
        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        // Not all platforms (WebGPU) support sRGB swapchains
        let view_format = config.format.add_srgb_suffix();
        config.view_formats.push(view_format);
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
        // let swapchain_capabilities = surface.get_capabilities(&adapter);
        // let swapchain_format = swapchain_capabilities.formats[0];
        let stencil_s = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Invert,
            pass_op: wgpu::StencilOperation::Invert,
        };
        let stencil_r = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep,
        };
        let spipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Stencil Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: VERTEX_STRUCT_SIZE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "stencil",
                compilation_options: Default::default(),
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: stencil_s,
                    back: stencil_s,
                    read_mask: 1,
                    write_mask: 1,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            multiview: None,
            cache: None,
        });
        let rpipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: VERTEX_STRUCT_SIZE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.view_formats[0],
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState {
                    front: stencil_r,
                    back: stencil_r,
                    read_mask: 1,
                    write_mask: 1,
                },
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
            spipeline,
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
            cpass.dispatch_workgroups((((POS.len() / 2) as f32) / 64.0).ceil() as u32, 1, 1);
        }
        let indices = self.cpipeline.ind_buff.size() as u32 / 4;
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Stencil Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: None,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.spipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.cpipeline.vert_buff.slice(..));
            rpass.set_index_buffer(self.cpipeline.ind_buff.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..indices, 0, 0..1);
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
            rpass.set_stencil_reference(1);
            rpass.draw_indexed(0..indices, 0, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));

        frame.present();

        // let staging_buffer = staging_buffer.slice(..);
        // staging_buffer.map_async(wgpu::MapMode::Read, |_| {});
        // self.device.poll(wgpu::Maintain::Wait);
        // let v = staging_buffer.get_mapped_range();
        // let buffer = StorageBuffer::new(v.as_ref());
        // let mut p: Vec<u32> = Vec::new();
        // buffer.read(&mut p).unwrap();

        // // let v: Vec<Vertex> = bytemuck::cast_slice(&v).to_vec();
        // println!("{:#?}", p);
        // // staging_buffer.;
        // // panic!("");

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
