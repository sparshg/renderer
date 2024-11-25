mod compute;
mod geometry;
mod renderer;
mod texture;

use cgmath::Vector3;
use geometry::shapes::{Circle, Square};
use renderer::{Scene, SurfaceContext};
use winit::event::WindowEvent;

pub const VERTEX_STRUCT_SIZE: u64 = 32;

struct State {
    scene: Scene,
}

impl State {
    fn new(ctx: &SurfaceContext<'_>) -> Self {
        let mut scene = Scene::new(ctx);
        let q1 = Square::new(1.);
        let q2 = Circle::new(1.);
        // q1.color(Vector4::new(0.8, 0.05, 0.05, 0.9));
        // q1.scale(Vector3::new(0.5, 0.5, 0.5));
        // q1.shift(Vector3::new(-0.1, 0., 0.));
        // q2.scale(Vector3::new(0.5, 0.5, 0.5));
        add!(scene, ctx, q1);
        // let q1 = scene.add(ctx, q1);

        scene.modify(q1, |q| {
            q.shift(Vector3::new(0.0, 0.0, 0.));
        });
        // scene.modify(q2, |q| {
        //     q.shift(Vector3::new(0.0, 0.0, 0.));
        // });
        // let mut q1 = QBezier::square();
        // let mut q1 = QBezier::quadratic_bezier_points_for_arc(2. * PI, 8);
        // let mut q2 = QBezier::quadratic_bezier_points_for_arc(2. * PI, 16);
        // q2.color(Vector4::new(0.05, 0.8, 0.05, 0.9));
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
