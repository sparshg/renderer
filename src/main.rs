mod animations;
mod core;
mod geometry;
mod texture;

// use animations::{Animation, Transformation};
use animations::Transformation;
use cgmath::{Deg, Quaternion, Rad, Rotation2, Rotation3};
use core::{HasPoints, Scene, SurfaceContext};
use geometry::shapes::{Square, Triangle};
use std::{
    cell::RefCell,
    f32::consts::PI,
    ops::Deref,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
    time::Instant,
};
use tokio::task::LocalSet;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

struct State<'a> {
    scene: Scene,
    ctx: SurfaceContext<'a>,
}

impl<'a> State<'a> {
    fn new(ctx: SurfaceContext<'a>) -> Self {
        let scene = Scene::new(&ctx);
        Self { scene, ctx }
    }

    fn construct(&mut self) {
        let q1 = geometry::shapes::Arc::circle(1.);
        q1.shift((0.0, 0.0, 0.0)).scale(0.5);

        let q2 = Square::new(1.);
        q2.shift((0.0, 0.0, 0.0)).color((0.8, 0.95, 0.05, 0.9));

        let q3 = Triangle::new(1.);
        q3.shift((0.0, 0.0, 0.0)).color((0.8, 0.05, 0.05, 0.9));

        let q = q1.clone();
        self.scene.add(&self.ctx, &q);
        self.scene.play(Transformation::new(&q, &q2, 1.));
        self.scene.play(Transformation::new(&q, &q3, 1.));
        self.scene.play(Transformation::new(&q, &q1, 1.));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.ctx.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.scene.render(&self.ctx, &view);
        frame.present();

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.ctx.resize(new_size);
        self.scene.camera.aspect = self.ctx.config.width as f32 / self.ctx.config.height as f32;
        self.scene.depth_texture = texture::Texture::create_depth_texture(
            &self.ctx.device,
            (self.ctx.config.width, self.ctx.config.height),
            "depth_texture",
        );
        self.update(std::time::Duration::from_secs(0));
    }

    fn input(&mut self, event: &WindowEvent) {
        self.scene.camera.process_inputs(event)
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.scene.update(&self.ctx, dt);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Rc::new(
        winit::window::WindowBuilder::new()
            .with_title("wgpu")
            .build(&event_loop)
            .unwrap(),
    );

    env_logger::init();
    let ctx = core::Context::init().await.attach_window(&window);
    let mut app = State::new(ctx);
    app.construct();

    let window = window.clone();
    let mut last_render_time = Instant::now();
    event_loop
        .run(move |event, target| {
            let Event::WindowEvent { event, .. } = event else {
                return;
            };
            app.input(&event);

            match event {
                WindowEvent::Resized(new_size) => {
                    // ctx.resize(new_size);
                    app.resize(new_size);
                    window.request_redraw();
                }
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::RedrawRequested => {
                    window.request_redraw();
                    let now = Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    app.update(dt);
                    match app.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            log::error!("Surface lost or outdated");
                            target.exit();
                            // app.resize(app.size);
                        }

                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("OutOfMemory");
                            target.exit();
                        }

                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }
                _ => {}
            };
        })
        .unwrap();
}
