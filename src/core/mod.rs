mod camera;
mod renderer;
mod shape;
mod utils;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;

use crate::animations::Animation;
use crate::texture::Texture;
use camera::Camera;
use cgmath::Matrix4;
use cgmath::SquareMatrix;
use cgmath::Vector4;
use encase::ShaderType;
use renderer::QBezierRenderer;
pub use shape::HasPoints;
pub use shape::Mobject;
pub use shape::Renderable;
pub use shape::Shape;
use shape::Transform;
use tokio::sync::oneshot;
pub use utils::bindgroup::{Attach, BindGroupBuilder};
pub use utils::context::AnyContext;
pub use utils::context::Context;
pub use utils::context::SurfaceContext;
pub use utils::pipeline::PipelineBuilder;

#[derive(Debug, ShaderType, Clone)]
pub struct ObjectUniforms {
    pub model: Matrix4<f32>,
    pub color: Vector4<f32>,
}

impl Default for ObjectUniforms {
    fn default() -> Self {
        Self {
            model: Matrix4::identity(),
            color: Vector4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl ObjectUniforms {
    pub fn new(transform: &Transform, color: Vector4<f32>) -> Self {
        Self {
            model: transform.get_matrix(),
            color,
        }
    }
}

// impl ObjectUniforms {
//     pub fn lerp(&self, other: &Self, t: f32) -> Self {
//         Self {
//             model: self.model.lerp(other.model, t),
//             color: self.color.lerp(other.color, t),
//         }
//     }
// }

#[derive(Clone)]
pub struct Scene<'a> {
    inner: Rc<RefCell<InnerScene<'a>>>,
}

impl<'a> Deref for Scene<'a> {
    type Target = Rc<RefCell<InnerScene<'a>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> Scene<'a> {
    pub fn new(ctx: SurfaceContext<'a>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(InnerScene::new(ctx))),
        }
    }

    pub fn add<T: HasPoints + 'static>(&self, shape: &Mobject<T>) {
        self.inner.borrow_mut().add(shape);
    }

    pub fn remove<T: HasPoints + 'static>(&self, shape: Mobject<T>) {
        self.inner.borrow_mut().remove(shape);
    }

    pub async fn play(&self, anim: impl Animation + 'static) {
        let rx = self.inner.borrow_mut().play(anim);
        rx.await.unwrap();
    }
}

pub struct InnerScene<'a> {
    ctx: SurfaceContext<'a>,
    camera: Camera,
    depth_texture: Texture,
    objects: Vec<Rc<RefCell<dyn Renderable>>>,
    animation: Option<(Box<dyn Animation>, oneshot::Sender<()>)>,
    qbezier_renderer: QBezierRenderer,
    t: f32,
    // mesh_renderer: MeshRenderer,
}

#[macro_export]
macro_rules! add {
    ($scene:ident, $($shape:ident),*) => {
        $(
            $scene.add(&$shape);
        )*
    };
}

#[macro_export]
macro_rules! remove {
    ($scene:ident, $($shape:ident),*) => {
        $(
            $scene.remove($shape);
        )*
    };
}

impl<'a> InnerScene<'a> {
    fn new(ctx: SurfaceContext<'a>) -> Self {
        let depth_texture = Texture::create_depth_texture(
            &ctx.device,
            (ctx.config.width, ctx.config.height),
            "Depth Texture",
        );
        let camera = Camera::new(&ctx);
        Self {
            objects: Vec::new(),
            qbezier_renderer: QBezierRenderer::new(&ctx, &camera.bind_group_layout),
            depth_texture,
            camera,
            animation: None,
            ctx,
            t: 0.,
        }
    }

    fn upcast<T: HasPoints + 'static>(shape: Rc<RefCell<Shape<T>>>) -> Rc<RefCell<dyn Renderable>> {
        shape
    }

    fn add<T: HasPoints + 'static>(&mut self, shape: &Mobject<T>) {
        shape
            .borrow_mut()
            .create_render_object(&self.ctx, self.qbezier_renderer.render_layout());
        self.objects.push(shape.deref().clone());
    }

    fn remove<T: HasPoints + 'static>(&mut self, shape: Mobject<T>) {
        // TODO: This is O(n)
        self.objects
            .retain(|x| Rc::ptr_eq(x, &Self::upcast(shape.clone())));
    }

    pub fn update(&mut self, dt: Duration) {
        self.camera.update_camera(&self.ctx);
        if let Some((anim, _)) = self.animation.as_mut() {
            if !anim.apply(self.t) {
                let (_, tx) = self.animation.take().unwrap();
                tx.send(()).unwrap();
            }
            self.t += dt.as_secs_f32();
        }
    }

    fn play(&mut self, mut anim: impl Animation + 'static) -> oneshot::Receiver<()> {
        self.t = 0.;
        anim.begin();
        let (tx, rx) = oneshot::channel();
        self.animation = Some((Box::new(anim), tx));
        rx
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.ctx.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .ctx
            .device()
            .create_command_encoder(&Default::default());

        for object in &self.objects {
            self.qbezier_renderer.render(
                &self.ctx,
                &view,
                &self.depth_texture.view,
                &self.camera.bind_group,
                &mut encoder,
                object,
                false,
            );
        }

        self.ctx.queue().submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.ctx.resize(new_size);
        self.camera.aspect = self.ctx.config.width as f32 / self.ctx.config.height as f32;
        self.depth_texture = Texture::create_depth_texture(
            &self.ctx.device,
            (self.ctx.config.width, self.ctx.config.height),
            "depth_texture",
        );
    }

    pub fn process_inputs(&mut self, event: &winit::event::WindowEvent) {
        self.camera.process_inputs(event);
    }
}
