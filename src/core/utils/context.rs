
use winit::dpi::PhysicalSize;

pub trait AnyContext {
    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
}

macro_rules! impl_context {
    ($type:ty) => {
        impl AnyContext for $type {
            fn device(&self) -> &wgpu::Device {
                &self.device
            }
            fn queue(&self) -> &wgpu::Queue {
                &self.queue
            }
        }
    };
}

impl_context!(Context);
impl_context!(SurfaceContext<'_>);

pub struct Context {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Context {
    pub async fn init() -> Self {
        log::info!("Initializing wgpu context...");

        let instance = wgpu::Instance::default();
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, None)
            .await
            .expect("No suitable GPU adapters found on the system!");
        let adapter_info = adapter.get_info();
        log::info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .expect("Unable to find a suitable GPU adapter!");
        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    pub fn attach_window(self, window: &winit::window::Window) -> SurfaceContext<'_> {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        let surface = self.instance.create_surface(window).unwrap();

        let mut config = surface
            .get_default_config(&self.adapter, size.width, size.height)
            .expect("Surface isn't supported by the adapter.");

        // Not all platforms (WebGPU) support sRGB swapchains, so we need to use view formats
        let view_format = config.format.add_srgb_suffix();
        config.view_formats.push(view_format);

        surface.configure(&self.device, &config);
        SurfaceContext {
            device: self.device,
            queue: self.queue,
            surface,
            config,
        }
    }
}

pub struct SurfaceContext<'a> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'a>,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'a> SurfaceContext<'a> {
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        log::info!("Surface resize {size:?}");

        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }
}
