use cgmath::{Basis2, Basis3, Matrix4, Quaternion, Rotation2, Rotation3, Vector2, Vector3};

struct Transform {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
}

struct Object {
    transform: Transform,
}

// impl Object {
//     fn new(points: Vec<Vector2<f32>>) -> Self {
//         Self {
//             points,
//             rotation: Basis3::from(Matrix4::)
//         }
//     }
// }
