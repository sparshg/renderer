mod animations;
mod core;
mod geometry;
mod texture;

// use animations::{Animation, Transformation};
use animations::Transformation;
use cgmath::{Deg, Quaternion, Rad, Rotation2, Rotation3};
use core::{HasPoints, Scene, SurfaceContext};
use geometry::shapes::{Arc, Square};
use std::{f32::consts::PI, ops::Deref};
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

        // let q1 = Square::new(1.);
        // q1.shift((0.0, 0.0, 0.0))
        let q2 = Square::new(1.);
        q2.shift((0.0, 0.0, 0.0)).color((0.8, 0.95, 0.05, 0.9));

        // .rotate(Quaternion::from_angle_z(Deg(45.)));
        // q1.borrow().points
        // let mut q3 = q1.clone();
        // q3.interpolate(&q1, &q2, 0.2);
        // q1.borrow_mut().radius = 0.1;
        // q1.shift((1.0, 0.0, 0.0)).color((0.8, 0.05, 0.05, 0.9));
        // scene.add(ctx, &q1);
        scene.add(ctx, &q1);
        let anim = Transformation::new(&q1, &q2, 1.);
        // panic!();
        scene.animations.push(Box::new(anim));
        // scene.remove(q2);
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
        self.update(ctx, std::time::Duration::from_secs(0));
    }

    fn input(&mut self, event: &WindowEvent) {
        self.scene.camera.process_inputs(event)
    }

    fn update(&mut self, ctx: &SurfaceContext, dt: std::time::Duration) {
        self.scene.update(ctx, dt);
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
