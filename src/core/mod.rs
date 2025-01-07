mod camera;
mod renderer;
mod shape;
mod utils;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;

use crate::animations::Animation;
use camera::Camera;
use cgmath::Matrix4;
use cgmath::SquareMatrix;
use cgmath::Vector4;
use encase::ShaderType;
use renderer::QBezierRenderer;
pub use shape::HasPoints;
use shape::Transform;
pub use utils::bindgroup::{Attach, BindGroupBuilder};
pub use utils::context::AnyContext;
pub use utils::context::Context;
pub use utils::context::SurfaceContext;
pub use utils::pipeline::PipelineBuilder;
// use crate::animations::Animation;
use crate::texture::Texture;
pub use shape::Mobject;
pub use shape::Renderable;
pub use shape::Shape;

// pub enum PipelineType {
//     QBezier,
//     // Mesh,
// }

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

pub struct Scene {
    pub camera: Camera,
    pub depth_texture: Texture,
    pub objects: Vec<Rc<RefCell<dyn Renderable>>>,
    pub animations: VecDeque<Box<dyn Animation>>,
    pub qbezier_renderer: QBezierRenderer,
    t: f32,
    // mesh_renderer: MeshRenderer,
}

#[macro_export]
macro_rules! add {
    ($scene:ident, $ctx:ident, $($shape:ident),*) => {
        $(
            $scene.add($ctx, $shape);
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

impl Scene {
    pub fn new(ctx: &SurfaceContext) -> Self {
        let depth_texture = Texture::create_depth_texture(
            &ctx.device,
            (ctx.config.width, ctx.config.height),
            "Depth Texture",
        );
        let camera = Camera::new(ctx);
        Self {
            objects: Vec::new(),
            qbezier_renderer: QBezierRenderer::new(ctx, &camera.bind_group_layout),
            depth_texture,
            camera,
            animations: VecDeque::new(),
            t: 0.,
        }
    }

    fn upcast<T: HasPoints + 'static>(shape: Rc<RefCell<Shape<T>>>) -> Rc<RefCell<dyn Renderable>> {
        shape
    }

    pub fn add<T: HasPoints + 'static>(&mut self, ctx: &SurfaceContext, shape: &Mobject<T>) {
        shape
            .borrow_mut()
            .create_render_object(ctx, self.qbezier_renderer.render_layout());
        self.objects.push(shape.deref().clone());
    }

    pub fn remove<T: HasPoints + 'static>(&mut self, shape: Mobject<T>) {
        // TODO: This is O(n)
        self.objects
            .retain(|x| Rc::ptr_eq(x, &Self::upcast(shape.clone())));
    }

    pub fn update(&mut self, ctx: &SurfaceContext, dt: Duration) {
        self.camera.update_camera(ctx);
        if let Some(anim) = self.animations.front_mut() {
            if self.t == 0. {
                anim.begin();
            }
            anim.apply(self.t);
            self.t += dt.as_secs_f32();
        }
        if self.t > 1. {
            self.animations.pop_front();
            self.t = 0.;
        }
    }

    pub fn play(&mut self, anim: impl Animation + 'static) {
        self.animations.push_back(Box::new(anim));
    }

    pub fn render(&mut self, ctx: &SurfaceContext, view: &wgpu::TextureView) {
        let mut encoder = ctx.device().create_command_encoder(&Default::default());

        for object in &self.objects {
            self.qbezier_renderer.render(
                ctx,
                view,
                &self.depth_texture.view,
                &self.camera.bind_group,
                &mut encoder,
                object,
                false,
            );
        }

        ctx.queue().submit(std::iter::once(encoder.finish()));
    }
}
