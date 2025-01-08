mod animations;
mod core;
mod geometry;
mod texture;

use animations::Transformation;
use core::{Scene, SurfaceContext};
use futures::executor::LocalPool;
use futures::task::LocalSpawnExt;
use geometry::shapes::{Square, Triangle};
use std::{
    cell::{Ref, RefCell, RefMut},
    ops::Deref,
    rc::Rc,
    time::Instant,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

async fn construct(scene: Rc<RefCell<Scene>>, ctx: Rc<RefCell<SurfaceContext<'_>>>) {
    let q1 = geometry::shapes::Arc::circle(1.);
    q1.shift((0.0, 0.0, 0.0)).scale(0.5);

    let q2 = Square::new(1.);
    q2.shift((0.0, 0.0, 0.0)).color((0.8, 0.95, 0.05, 0.9));

    let q3 = Triangle::new(1.);
    q3.shift((0.0, 0.0, 0.0)).color((0.8, 0.05, 0.05, 0.9));

    let q = q1.clone();
    scene.borrow_mut().add(&ctx.borrow(), &q);
    let k = scene.borrow_mut().play(Transformation::new(&q, &q2, 1.));
    k.await.unwrap();
    let r = scene.borrow_mut().play(Transformation::new(&q, &q3, 1.));
    r.await.unwrap();
}

struct State<'a> {
    scene: Rc<RefCell<Scene>>,
    ctx: Rc<RefCell<SurfaceContext<'a>>>,
}

impl<'a> State<'a> {
    fn new(ctx: SurfaceContext<'a>) -> Self {
        let scene = Rc::new(RefCell::new(Scene::new(&ctx)));
        Self {
            scene,
            ctx: Rc::new(RefCell::new(ctx)),
        }
    }

    fn scene(&self) -> RefMut<'_, Scene> {
        self.scene.borrow_mut()
    }

    fn ctx(&self) -> Ref<'_, SurfaceContext<'a>> {
        self.ctx.borrow()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.ctx().surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.scene().render(&self.ctx(), &view);
        frame.present();

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.ctx.borrow_mut().resize(new_size);
        self.scene().camera.aspect =
            self.ctx().config.width as f32 / self.ctx().config.height as f32;
        self.scene().depth_texture = texture::Texture::create_depth_texture(
            &self.ctx().device,
            (self.ctx().config.width, self.ctx().config.height),
            "depth_texture",
        );
    }

    fn input(&mut self, event: &WindowEvent) {
        self.scene().camera.process_inputs(event)
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.scene().update(&self.ctx(), dt);
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
    let ctx: SurfaceContext<'_>;
    unsafe {
        let window: *const Window = window.deref();
        ctx = core::Context::init()
            .await
            .attach_window(window.as_ref().unwrap());
    }
    let mut app = State::new(ctx);

    let mut local_pool = LocalPool::new();
    let spawner = local_pool.spawner();

    spawner
        .spawn_local(construct(app.scene.clone(), app.ctx.clone()))
        .expect("Failed to spawn");

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
                    app.resize(new_size);
                    window.request_redraw();
                }
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::RedrawRequested => {
                    window.request_redraw();
                    let now = Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    local_pool.run_until_stalled();
                    app.update(dt);
                    match app.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            log::error!("Surface lost or outdated");
                            target.exit();
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
