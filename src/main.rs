mod camera;
mod compute;
mod object;
mod renderer;
mod texture;

use cgmath::{Rotation3, Vector3, Vector4};
use compute::POS;
use object::QBezier;
use renderer::{Renderer, SurfaceContext};
use winit::event::WindowEvent;

pub const VERTEX_STRUCT_SIZE: u64 = 32;

struct State {
    renderer: Renderer,
    qbezier: Vec<QBezier>,
}

impl State {
    async fn new(ctx: &SurfaceContext<'_>) -> Self {
        let mut q1 = QBezier::new(POS.map(Vector3::from).into_iter().collect());
        let mut q2 = QBezier::new(POS.map(Vector3::from).into_iter().collect());
        q1.color(Vector4::new(1.0, 0.0, 0.0, 1.0));
        // q2.color(Vector4::new(0.0, 1.0, 0.0, 1.0));
        q1.shift(Vector3::new(-0.4, 0., 0.));
        // q2.shift(Vector3::new(1.0, 0.0, 0.0));
        Self {
            renderer: Renderer::new(ctx).await,
            qbezier: vec![q1, q2],
        }
    }
}

impl renderer::App for State {
    fn render(&mut self, ctx: &SurfaceContext) -> Result<(), wgpu::SurfaceError> {
        let frame = ctx.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        for qbezier in &mut self.qbezier {
            self.renderer
                .render_qbezier(ctx, &view, &mut encoder, qbezier);
        }
        ctx.queue.submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }

    fn resize(&mut self, ctx: &mut SurfaceContext, new_size: winit::dpi::PhysicalSize<u32>) {
        ctx.resize(new_size);
        self.renderer.camera.aspect = ctx.config.width as f32 / ctx.config.height as f32;
        self.renderer.depth_texture = texture::Texture::create_depth_texture(
            &ctx.device,
            (ctx.config.width, ctx.config.height),
            "depth_texture",
        );
        self.update(ctx);
    }

    fn input(&mut self, event: &WindowEvent) {
        self.renderer.camera.process_inputs(event)
    }

    fn update(&mut self, ctx: &SurfaceContext) {
        self.renderer.camera.update_camera(ctx);
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
