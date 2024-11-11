// use context::SurfaceWrapper;

mod bindgroup;
mod context;
mod pipeline;
mod window;

pub use bindgroup::BindGroupBuilder;
pub use context::Context;
pub use context::SurfaceContext;
pub use pipeline::Pipeline;
pub use pipeline::PipelineBuilder;
pub use window::App;
pub use window::Window;

async fn test() {
    let win = window::Window::new("test");
    let w = win.get_window();
    let mut ctx = context::Context::init().await.attach_window(&w);

    // // let SurfaceWrapper { surface, config } = window::Window::init_surface(&ctx, &win.window);
    // // ctx.surface = Some(surface);
    // // ctx.config = Some(config);
    let shader = ctx
        .device
        .create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
    let targets: [Option<wgpu::ColorTargetState>; 0] = [];
    // let rp = pipeline::RenderPipelineBuilder::for_render(Some("label"), &module);
    // let rpipeline = PipelineBuilder::for_render("Render Pipeline", &shader)
    //     .vertex(&[wgpu::VertexBufferLayout {
    //         array_stride: 32,
    //         step_mode: wgpu::VertexStepMode::Vertex,
    //         attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
    //     }])
    //     .fragment("fs_main")
    //     .depth_stencil(true, stencil_r, 1, 1)
    //     .build(&ctx);
}
