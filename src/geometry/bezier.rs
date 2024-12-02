use super::Transform;
use crate::renderer::{
    AnyContext, Attach, ComputeObject, ObjectUniforms, RenderObject, SurfaceContext,
};
use cgmath::{ElementWise, Quaternion, Vector3, Vector4};
use wgpu::{util::DeviceExt, BufferAddress};

pub struct QBezierPath {
    pub points: Vec<Vector3<f32>>,
    pub uniforms: ObjectUniforms,
    pub render_object: Option<RenderObject>,
    pub compute_object: Option<ComputeObject>,
}

impl Clone for QBezierPath {
    fn clone(&self) -> Self {
        Self {
            points: self.points.clone(),
            uniforms: self.uniforms.clone(),
            render_object: None,
            compute_object: None,
        }
    }
}

impl QBezierPath {
    const VERTEX_SIZE: usize = 32;

    pub fn new(points: Vec<Vector3<f32>>) -> Self {
        Self {
            points,
            uniforms: ObjectUniforms::default(),
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

    pub fn create_render_buffers(&mut self, ctx: &SurfaceContext, layout: &wgpu::BindGroupLayout) {
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
            vec![uniform_buffer.as_entire_binding()],
        );

        self.render_object = Some(RenderObject {
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            // renderer_type: PipelineType::QBezier,
        });
    }

    pub fn update_compute_buffers(
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
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
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

    pub fn update_render_buffers(&self, ctx: &SurfaceContext, uniforms: &ObjectUniforms) {
        let render_object = self
            .render_object
            .as_ref()
            .expect("Render Object not initialized. Create render buffers first");

        let mut data = encase::UniformBuffer::new(Vec::new());
        data.write(uniforms).unwrap();
        let data = data.into_inner();

        ctx.queue()
            .write_buffer(&render_object.uniform_buffer, 0, &data);
    }

    pub fn num_compute_workgroups(&self) -> u32 {
        (((self.points.len() / 2) as f32) / 64.0).ceil() as u32
    }
}
