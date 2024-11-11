mod camera;
mod compute;
mod object;
mod renderer;
mod texture;

use camera::{Camera, CameraUniform};
use compute::POS;
use renderer::{BindGroupBuilder, PipelineBuilder, SurfaceContext};
use texture::Texture;
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout};
use winit::event::WindowEvent;

pub const VERTEX_STRUCT_SIZE: u64 = 32;

struct State {
    rpipeline: renderer::Pipeline,
    spipeline: renderer::Pipeline,
    cpipeline: compute::ComputePipeline,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    diffuse_bind_group: wgpu::BindGroup,
    depth_texture: Texture,
}

impl State {
    async fn new(ctx: &SurfaceContext<'_>) -> Self {
        let diffuse_texture = texture::Texture::from_bytes(
            &ctx.device,
            &ctx.queue,
            include_bytes!("happy-tree.png"),
            "happy-tree.png",
        )
        .unwrap();

        let texture_bind_group_layout = BindGroupBuilder::new("texture_bind_group_layout")
            .add_texture_float_filterable_d2(wgpu::ShaderStages::FRAGMENT, false)
            .add_sampler_filterable(wgpu::ShaderStages::FRAGMENT)
            .build(ctx);
        let diffuse_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let camera_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout = BindGroupBuilder::new("camera_bind_group_layout")
            .add_uniform_buffer(wgpu::ShaderStages::VERTEX, None)
            .build(ctx);

        let camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
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
        let spipeline = PipelineBuilder::for_render("Stencil Pipeline", &shader)
            .vertex(&[wgpu::VertexBufferLayout {
                array_stride: VERTEX_STRUCT_SIZE,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
            }])
            .fragment("stencil", &[])
            .depth_stencil(false, stencil_s, 1, 1)
            .add_bind_group_layout(&camera_bind_group_layout)
            .add_bind_group_layout(&texture_bind_group_layout)
            .build(ctx);

        let rpipeline = PipelineBuilder::for_render("Render Pipeline", &shader)
            .vertex(&[wgpu::VertexBufferLayout {
                array_stride: VERTEX_STRUCT_SIZE,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
            }])
            .fragment(
                "fs_main",
                &[Some(wgpu::ColorTargetState {
                    format: ctx.config.view_formats[0],
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            )
            .depth_stencil(true, stencil_r, 1, 1)
            .add_bind_group_layout(&camera_bind_group_layout)
            .add_bind_group_layout(&texture_bind_group_layout)
            .build(ctx);

        let cpipeline = compute::ComputePipeline::new(&ctx);

        let depth_texture = texture::Texture::create_depth_texture(
            &ctx.device,
            (ctx.config.width, ctx.config.height),
            "depth_texture",
        );
        Self {
            rpipeline,
            spipeline,
            cpipeline,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            diffuse_bind_group,
            depth_texture,
        }
    }
}

impl renderer::App for State {
    fn render(&self, ctx: &SurfaceContext) -> Result<(), wgpu::SurfaceError> {
        let frame = ctx.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
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
        ctx.queue.submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }

    fn resize(&mut self, ctx: &mut SurfaceContext, new_size: winit::dpi::PhysicalSize<u32>) {
        ctx.resize(new_size);
        self.camera.aspect = ctx.config.width as f32 / ctx.config.height as f32;
        self.depth_texture = texture::Texture::create_depth_texture(
            &ctx.device,
            (ctx.config.width, ctx.config.height),
            "depth_texture",
        );
        self.update(ctx);
    }

    fn input(&mut self, event: &WindowEvent) {
        self.camera.process_inputs(event)
    }

    fn update(&mut self, ctx: &SurfaceContext) {
        self.camera.update_camera();
        self.camera_uniform.update_view_proj(&self.camera);
        ctx.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}

async fn run() {
    let window = renderer::Window::new("wgpu");
    env_logger::init();
    let w = window.get_window();
    let mut ctx = renderer::Context::init().await.attach_window(&w);
    let app = State::new(&ctx).await;
    window.run(&mut ctx, app);
}
pub fn main() {
    pollster::block_on(run());
}
