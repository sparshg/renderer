use std::f32::consts::PI;

use super::{Path, Shape, ShapeBuffer, Transform};
use crate::renderer::{
    AnyContext, Attach, ComputeObject, ObjectUniforms, RenderObject, SurfaceContext,
};
use cgmath::{ElementWise, Quaternion, Vector3, Vector4};
use encase::ShaderType;
use wgpu::{util::DeviceExt, BufferAddress};

pub struct QBezierPath {
    points: Vec<Vector3<f32>>,
    transform: Transform,
    pub render_object: Option<RenderObject>,
    pub compute_object: Option<ComputeObject>,
}

impl QBezierPath {
    const VERTEX_SIZE: usize = 32;

    fn new(points: Vec<Vector3<f32>>) -> Self {
        Self {
            points,
            transform: Transform::new(),
            compute_object: None,
            render_object: None,
        }
    }

    fn create_vertex_buffer(&self, ctx: &SurfaceContext) -> wgpu::Buffer {
        ctx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (self.points.len() as u64 / 2 * 3 + 1) * Self::VERTEX_SIZE as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_index_buffer(&self, ctx: &SurfaceContext) -> wgpu::Buffer {
        ctx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (self.points.len() as u64 / 2 * 6)
                * std::mem::size_of::<u32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn circle() -> Self {
        let angle = 2. * PI;
        let n_components = 8;
        let n_points = 2 * n_components + 1;
        let angles = (0..n_points).map(|i| i as f32 * angle / (n_points - 1) as f32);
        let mut points = angles
            .map(|angle| Vector3::new(angle.cos(), angle.sin(), 0.))
            .collect::<Vec<_>>();
        let theta = angle / n_components as f32;
        let handle_adjust = 1.0 / (theta / 2.0).cos();

        for i in (1..n_points).step_by(2) {
            points[i as usize] *= handle_adjust;
        }
        Self::new(points)
    }

    pub fn square() -> Self {
        let points = vec![
            Vector3::new(0.5, 0.5, 0.),
            Vector3::new(0., 0.5, 0.),
            Vector3::new(-0.5, 0.5, 0.),
            Vector3::new(-0.5, 0., 0.),
            Vector3::new(-0.5, -0.5, 0.),
            Vector3::new(0., -0.5, 0.),
            Vector3::new(0.5, -0.5, 0.),
            Vector3::new(0.5, 0., 0.),
            Vector3::new(0.5, 0.5, 0.),
        ];
        Self::new(points)
    }
}

impl ShapeBuffer for QBezierPath {
    fn create_render_buffers(&mut self, ctx: &SurfaceContext, layout: &wgpu::BindGroupLayout) {
        let index_buffer = self.create_index_buffer(ctx);
        let vertex_buffer = self.create_vertex_buffer(ctx);
        let uniforms = ObjectUniforms::default();

        let mut buff = encase::UniformBuffer::new(Vec::<u8>::new());
        buff.write(&uniforms).unwrap();
        let buff = buff.into_inner();

        let uniform_buffer = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("QBezier Uniform Buffer"),
                contents: bytemuck::cast_slice(&buff),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = layout.attach(
            ctx,
            "QBezier Bind Group",
            vec![
                vertex_buffer.as_entire_binding(),
                index_buffer.as_entire_binding(),
                uniform_buffer.as_entire_binding(),
            ],
        );

        self.render_object = Some(RenderObject {
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            update: true,
            uniforms,
            // renderer_type: PipelineType::QBezier,
        });
    }

    fn update_compute_buffers(
        &mut self,
        ctx: &SurfaceContext,
        layout: &wgpu::BindGroupLayout,
    ) -> bool {
        if !self
            .compute_object
            .as_ref()
            .map(|o| o.update)
            .unwrap_or(true)
        {
            return false;
        }

        let mut data = encase::StorageBuffer::new(Vec::new());
        data.write(&self.points).unwrap();
        let data: Vec<u8> = data.into_inner();

        let mut reinit_point_buff = None;
        match self.compute_object.as_ref().map(|o| &o.buffer) {
            Some(b) if b.size() == data.len() as BufferAddress => {
                ctx.queue().write_buffer(b, 0, &data);
            }
            _ => {
                reinit_point_buff = Some(ctx.device().create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Point Buffer"),
                        usage: wgpu::BufferUsages::STORAGE,
                        contents: &data,
                    },
                ));
            }
        }

        if let Some(point_buffer) = reinit_point_buff {
            let vertex_buffer = self.create_vertex_buffer(ctx);
            let index_buffer = self.create_index_buffer(ctx);

            let render_object = self
                .render_object
                .as_mut()
                .expect("Render Object not initialized. Create render buffers first");
            render_object.vertex_buffer = vertex_buffer;
            render_object.index_buffer = index_buffer;

            self.compute_object = Some(ComputeObject {
                bind_group: layout.attach(
                    ctx,
                    "Compute Bind Group",
                    vec![
                        point_buffer.as_entire_binding(),
                        render_object.vertex_buffer.as_entire_binding(),
                        render_object.index_buffer.as_entire_binding(),
                    ],
                ),
                update: false,
                buffer: point_buffer,
            });
        }
        self.compute_object.as_mut().unwrap().update = false;
        true
    }

    fn update_render_buffers(&mut self, ctx: &SurfaceContext) {
        let render_object = self
            .render_object
            .as_mut()
            .expect("Render Object not initialized. Create render buffers first");
        if !render_object.update {
            return;
        }

        render_object.uniforms.model = self.transform.get_matrix();

        let mut data = encase::UniformBuffer::new(Vec::new());
        data.write(&render_object.uniforms).unwrap();
        let data = data.into_inner();

        render_object.update = false;
        ctx.queue()
            .write_buffer(&render_object.uniform_buffer, 0, &data);
    }

    fn num_compute_workgroups(&self) -> u32 {
        (((self.points.len() / 2) as f32) / 64.0).ceil() as u32
    }
}

impl Shape for QBezierPath {
    fn shift(&mut self, offset: Vector3<f32>) {
        self.transform.position += offset;
        self.render_object.as_mut().map(|o| o.update = true);
    }
    fn rotate(&mut self, rotation: Quaternion<f32>) {
        self.transform.rotation = rotation * self.transform.rotation;
        self.render_object.as_mut().map(|o| o.update = true);
    }
    fn scale(&mut self, scale: Vector3<f32>) {
        self.transform.scale.mul_assign_element_wise(scale);
        self.render_object.as_mut().map(|o| o.update = true);
    }
    fn color(&mut self, color: Vector4<f32>) {
        // self.uniforms.color = color.into();
        self.render_object.as_mut().map(|o| o.update = true);
    }
    // fn get_render_object(&self) -> Weak<RefCell<RenderObject>> {
    //     Rc::downgrade(
    //         self.render_object
    //             .as_ref()
    //             .expect("Render Object not initialized. Create render buffers first"),
    //     )
    // }
}

impl Path for QBezierPath {
    fn points(&self) -> &[Vector3<f32>] {
        &self.points
    }
}
