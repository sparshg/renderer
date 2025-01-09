mod animations;
mod core;
mod geometry;
mod texture;

use animations::Transformation;
use core::{Scene, SurfaceContext};
use futures::executor::LocalPool;
use futures::task::LocalSpawnExt;
use geometry::shapes::{Arc, Square, Triangle};
use std::{ops::Deref, rc::Rc, time::Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

async fn construct(scene: Scene<'_>) {
    let q1 = Arc::circle(1.);
    q1.shift((0.0, 0.0, 0.0)).scale(0.5);

    let q2 = Square::new(1.);
    q2.shift((0.0, 0.0, 0.0)).color((0.8, 0.95, 0.05, 0.9));

    let q3 = Triangle::new(1.);
    q3.shift((0.0, 0.0, 0.0)).color((0.8, 0.05, 0.05, 0.9));

    let q = q1.clone();
    scene.add(&q);
    scene.play(Transformation::new(&q, &q2, 1.)).await;
    scene.play(Transformation::new(&q, &q3, 2.)).await;
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
    let scene = Scene::new(ctx);

    let mut local_pool = LocalPool::new();
    local_pool
        .spawner()
        .spawn_local(construct(scene.clone()))
        .expect("Failed to spawn");

    let mut last_render_time = Instant::now();
    event_loop
        .run(move |event, target| {
            let Event::WindowEvent { event, .. } = event else {
                return;
            };
            scene.borrow_mut().process_inputs(&event);
            local_pool.run_until_stalled();

            match event {
                WindowEvent::Resized(new_size) => {
                    scene.borrow_mut().resize(new_size);
                    window.request_redraw();
                }
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::RedrawRequested => {
                    window.request_redraw();
                    let now = Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    scene.borrow_mut().update(dt);
                    match scene.borrow_mut().render() {
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
