mod animations;
mod geometry;
mod renderer;
mod texture;

use std::ops::Deref;

use geometry::{
    shapes::{Arc, Square},
    Shape,
};
use renderer::{Scene, SurfaceContext};
use winit::event::WindowEvent;

pub const VERTEX_STRUCT_SIZE: u64 = 32;

struct State {
    scene: Scene,
}

impl State {
    fn new(ctx: &SurfaceContext<'_>) -> Self {
        let mut scene: Scene = Scene::new(ctx);

        let q1 = Square::new(1.);
        let q2 = Arc::circle(1.);

        add!(scene, ctx, q1, q2);
        scene.modify(&q1, |q| {
            q.shift((1.0, 0.0, 0.0)).color((0.8, 0.05, 0.05, 0.9));
        });
        scene.modify(&q2, |q| {
            q.shift((-1.0, 0.0, 0.0)).scale(0.5);
        });

        let anim =
            animations::Animation::new(scene.id_to_qbezier(&q1), scene.id_to_qbezier(&q2), 1.0);
        // scene.add(anim. );

        Self { scene }
    }
}

impl renderer::App for State {
    fn render(&mut self, ctx: &SurfaceContext) -> Result<(), wgpu::SurfaceError> {
        let frame = ctx.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.scene.render(ctx, &view);
        frame.present();

        Ok(())
    }

    fn resize(&mut self, ctx: &mut SurfaceContext) {
        self.scene.camera.aspect = ctx.config.width as f32 / ctx.config.height as f32;
        self.scene.depth_texture = texture::Texture::create_depth_texture(
            &ctx.device,
            (ctx.config.width, ctx.config.height),
            "depth_texture",
        );
        self.update(ctx);
    }

    fn input(&mut self, event: &WindowEvent) {
        self.scene.camera.process_inputs(event)
    }

    fn update(&mut self, ctx: &SurfaceContext) {
        self.scene.camera.update_camera(ctx);
    }
}

async fn run() {
    let window = renderer::Window::new("wgpu");
    env_logger::init();
    let w = window.get_window();
    let mut ctx = renderer::Context::init().await.attach_window(&w);
    let app = State::new(&ctx);
    window.run(&mut ctx, app);
}
pub fn main() {
    pollster::block_on(run());
}
