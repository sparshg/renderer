
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

use super::context::SurfaceContext;

pub struct Window {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
}

impl Window {
    pub fn new(title: &str) -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .unwrap();

        env_logger::init();

        Self { event_loop, window }
    }

    pub async fn run(self, ctx: &mut SurfaceContext<'_>) {
        self.event_loop
            .run(move |event, target| {
                let Event::WindowEvent { event, .. } = event else {
                    return;
                };
                //             state.input(&event);

                match event {
                    WindowEvent::Resized(new_size) => {
                        ctx.resize(new_size);
                        self.window.request_redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::RedrawRequested => {
                        self.window.request_redraw();
                        //                     state.update();
                        //                     match state.render() {
                        //                         Ok(_) => {}
                        //                         Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        //                             state.resize(state.size);
                        //                         }

                        //                         Err(wgpu::SurfaceError::OutOfMemory) => {
                        //                             log::error!("OutOfMemory");
                        //                             target.exit();
                        //                         }

                        //                         Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                        //                     }
                    }
                    _ => {}
                };
            })
            .unwrap();
    }
}
