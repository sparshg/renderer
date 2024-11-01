use cgmath::{InnerSpace, Matrix3, Point3, Vector3};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Camera {
    eye: Point3<f32>,
    forward: Vector3<f32>,
    up: Vector3<f32>,
    pub aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    last_cursor_position: Option<(f64, f64)>,
    cursor_down: bool,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    -1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl Camera {
    pub fn new() -> Self {
        Camera {
            eye: Point3::new(0.0, 0.0, -2.0),
            forward: Vector3::new(0.0, 0.0, 1.0),
            up: Vector3::unit_y(),
            aspect: 1.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            speed: 0.02,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            last_cursor_position: None,
            cursor_down: false,
        }
    }

    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_to_rh(self.eye, self.forward, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn process_inputs(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        // true
                    }
                    KeyCode::ShiftLeft => {
                        self.is_down_pressed = is_pressed;
                        // true
                    }
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        // true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        // true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        // true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        // true
                    }
                    _ => (),
                }
            }
            WindowEvent::MouseInput { state, .. } => {
                self.cursor_down = if *state == ElementState::Pressed {
                    true
                } else {
                    self.last_cursor_position = None;
                    false
                };
            }
            WindowEvent::CursorMoved { position, .. } if self.cursor_down => {
                let dx = position.x - self.last_cursor_position.unwrap_or((position.x, 0.0)).0;
                let dy = position.y - self.last_cursor_position.unwrap_or((0.0, position.y)).1;
                let yaw = cgmath::Rad(dx as f32 * 0.004);
                let pitch = cgmath::Rad(dy as f32 * 0.004);

                let right = self.forward.cross(self.up).normalize();
                let rot = Matrix3::from_axis_angle(self.up, yaw)
                    * Matrix3::from_axis_angle(right, -pitch);
                self.last_cursor_position = Some((position.x, position.y));

                self.forward = rot * self.forward;
            }
            _ => (),
        }
    }

    pub fn update_camera(&mut self) {
        use cgmath::InnerSpace;
        if self.is_forward_pressed {
            self.eye += self.forward * self.speed;
        }
        if self.is_backward_pressed {
            self.eye -= self.forward * self.speed;
        }
        if self.is_left_pressed {
            let right = self.forward.cross(self.up).normalize();
            self.eye += right * self.speed;
        }
        if self.is_right_pressed {
            let right = self.forward.cross(self.up).normalize();
            self.eye -= right * self.speed;
        }
        if self.is_up_pressed {
            self.eye += self.up * self.speed;
        }
        if self.is_down_pressed {
            self.eye -= self.up * self.speed;
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
