use std::rc::Rc;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

use super::context::SurfaceContext;

pub trait App {
    fn resize(&mut self, ctx: &mut SurfaceContext, size: winit::dpi::PhysicalSize<u32>);
    fn render(&mut self, ctx: &SurfaceContext) -> Result<(), wgpu::SurfaceError>;
    fn update(&mut self, ctx: &SurfaceContext);
    fn input(&mut self, event: &WindowEvent);
}
pub struct Window {
    window: Rc<winit::window::Window>,
    pub event_loop: EventLoop<()>,
}

impl Window {
    pub fn new(title: &str) -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = Rc::new(
            winit::window::WindowBuilder::new()
                .with_title(title)
                .build(&event_loop)
                .unwrap(),
        );

        Self { event_loop, window }
    }

    pub fn get_window(&self) -> Rc<winit::window::Window> {
        Rc::clone(&self.window)
    }

    pub fn run<T: App>(self, ctx: &mut SurfaceContext<'_>, mut app: T) {
        self.event_loop
            .run(move |event, target| {
                let Event::WindowEvent { event, .. } = event else {
                    return;
                };
                app.input(&event);

                match event {
                    WindowEvent::Resized(new_size) => {
                        ctx.resize(new_size);
                        app.resize(ctx, new_size);
                        self.window.request_redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::RedrawRequested => {
                        self.window.request_redraw();
                        app.update(ctx);
                        match app.render(ctx) {
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
}
