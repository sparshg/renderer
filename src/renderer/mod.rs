// use context::SurfaceWrapper;

mod context;
mod window;

pub use context::Context;
pub use context::SurfaceContext;
pub use window::App;
pub use window::Window;

// async fn test() {
//     let win = window::Window::new("test");
//     let mut ctx = context::Context::init().await;

//     let SurfaceWrapper { surface, config } = window::Window::init_surface(&ctx, &win.window);
//     ctx.surface = Some(surface);
//     ctx.config = Some(config);
// }
