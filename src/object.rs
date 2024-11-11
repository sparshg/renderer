use cgmath::{Quaternion, Vector3};

struct Transform {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
}

struct Object {
    transform: Transform,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

// impl Object {
//     fn new(points: Vec<Vector2<f32>>) -> Self {
//         Self {
//             points,
//             rotation: Basis3::from(Matrix4::)
//         }
//     }
// }
