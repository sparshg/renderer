mod camera;
mod compute;
mod object;
mod renderer;
mod texture;

use camera::Camera;
use compute::POS;
use renderer::{BindGroupBuilder, PipelineBuilder, SurfaceContext};
use texture::Texture;
use wgpu::RenderPipeline;
use winit::event::WindowEvent;

pub const VERTEX_STRUCT_SIZE: u64 = 32;

struct State {
    rpipeline: renderer::Pipeline<RenderPipeline>,
    spipeline: renderer::Pipeline<RenderPipeline>,
    cpipeline: compute::ComputePipeline,
    camera: Camera,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: Texture,
}

impl State {
    async fn new(ctx: &SurfaceContext<'_>) -> Self {
        let camera = Camera::new(ctx);

        let camera_bind_group_layout = BindGroupBuilder::new("camera_bind_group_layout")
            .add_uniform_buffer(wgpu::ShaderStages::VERTEX, None)
            .build(ctx);

        let camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera.buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

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
            .build(ctx);

        let cpipeline = compute::ComputePipeline::new(ctx);

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
            camera_bind_group,
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
            cpass.set_pipeline(&self.cpipeline.pipeline.pipeline);
            cpass.set_bind_group(0, &self.cpipeline.bind_group, &[]);
            cpass.dispatch_workgroups((((POS.len() / 2) as f32) / 64.0).ceil() as u32, 1, 1);
        }

        self.spipeline
            .begin_pass("Stencil Pass")
            .add_bind_group(&self.camera_bind_group)
            .add_vertex_buffer(&self.cpipeline.vert_buff)
            .add_index_buffer(&self.cpipeline.ind_buff)
            .pass(
                &mut encoder,
                &[],
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: None,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
            );
        self.rpipeline
            .begin_pass("Render Pass")
            .add_bind_group(&self.camera_bind_group)
            .add_vertex_buffer(&self.cpipeline.vert_buff)
            .add_index_buffer(&self.cpipeline.ind_buff)
            .set_stencil_reference(1)
            .pass(
                &mut encoder,
                &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            );
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
        self.camera.update_camera(ctx);
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
