// use context::SurfaceWrapper;

mod context;
mod pipeline;
mod window;

pub use context::Context;
pub use context::SurfaceContext;
pub use window::App;
pub use window::Window;

async fn test() {
    let win = window::Window::new("test");
    // let mut ctx = context::Context::init()
    //     .await
    //     .attach_window(&win.get_window());

    // // let SurfaceWrapper { surface, config } = window::Window::init_surface(&ctx, &win.window);
    // // ctx.surface = Some(surface);
    // // ctx.config = Some(config);
    // let module = ctx
    //     .device
    //     .create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
    // let rp = pipeline::RenderPipelineBuilder::for_render(Some("label"), &module);
}
