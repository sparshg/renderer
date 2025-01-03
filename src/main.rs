mod animations;
mod core;
mod geometry;
mod texture;

// use animations::{Animation, Transformation};
use core::{Scene, SurfaceContext};
use geometry::shapes::{Arc, Square};
use winit::event::WindowEvent;

pub const VERTEX_STRUCT_SIZE: u64 = 32;

struct State {
    scene: Scene,
}

impl State {
    fn new(ctx: &SurfaceContext) -> Self {
        let mut scene: Scene = Scene::new(ctx);

        let q1 = Arc::circle(1.);
        q1.shift((0.0, 0.0, 0.0)).scale(0.5);

        let q2 = Square::new(1.);
        q2.shift((0.0, 0.0, 0.0)).color((0.8, 0.05, 0.05, 0.9));
        // q1.borrow().points
        // let mut q3 = q1.clone();
        // q3.interpolate(&q1, &q2, 0.2);
        // let mut anim = Transformation::new(&q1, &q2, 1.);
        // anim.curr.qbezier_mut().create_render_buffers(
        //     ctx,
        //     &scene
        //         .qbezier_renderer
        //         .render_pipeline
        //         .get_bind_group_layout(1),
        // );
        // scene.animations.push(Box::new(anim));
        scene.add(ctx, &q1);
        scene.add(ctx, &q2);
        q1.shift((1.0, 0.0, 0.0)).color((0.8, 0.05, 0.05, 0.9));
        // scene.modify(&q2, |q| {
        //     q.shift((-1.0, 0.0, 0.0)).scale(0.5);
        // });

        // scene.add(anim. );

        Self { scene }
    }
}

impl core::App for State {
    fn render(&mut self, ctx: &SurfaceContext) -> Result<(), wgpu::SurfaceError> {
        let frame = ctx.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.scene.render(ctx, &view);
        frame.present();

        Ok(())
    }

    fn resize(&mut self, ctx: &SurfaceContext) {
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
        self.scene.update(ctx);
    }
}

async fn run() {
    let window = core::Window::new("wgpu");
    env_logger::init();
    let w = window.get_window();
    let mut ctx = core::Context::init().await.attach_window(&w);
    let app = State::new(&ctx);
    window.run(&mut ctx, app);
}
pub fn main() {
    pollster::block_on(run());
}
