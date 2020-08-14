use glam::{Mat4, Vec3};
use winit::event::KeyboardInput;

use super::buffer::ManagedBuffer;
use super::event_handler::EventHandler;

struct Frustum {
    fov_y: f32,
    aspect_ratio: f32,
    znear: f32,
    zfar: f32,
}

pub struct Camera {
    position: Vec3,
    forward: Vec3,
    x_angle: f32,
    y_angle: f32,
    frustum: Frustum,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Vec3::zero(),
            forward: -Vec3::unit_z(),
            x_angle: 0.0,
            y_angle: 0.0,
            frustum: Frustum {
                fov_y: 90.0,
                aspect_ratio: width as f32 / height as f32,
                znear: 0.01,
                zfar: 100.0,
            },
        }
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(
            self.position,
            self.position + self.forward,
            glam::vec3(0.0, 1.0, 0.0),
        )
    }

    pub fn projection(&self) -> Mat4 {
        let f = 1.0 / (self.frustum.fov_y.to_radians() * 0.5).tan();
        let a = f / self.frustum.aspect_ratio;
        let b = self.frustum.zfar / (self.frustum.znear - self.frustum.zfar);
        let c = self.frustum.znear * b;

        Mat4::from_cols(
            glam::vec4(a, 0.0, 0.0, 0.0),
            glam::vec4(0.0, f, 0.0, 0.0),
            glam::vec4(0.0, 0.0, b, -1.0),
            glam::vec4(0.0, 0.0, c, 0.0),
        )
    }

    fn transforms_data(&self) -> Vec<f32> {
        let mut transforms_data = Vec::<f32>::new();
        transforms_data.extend(self.view().to_cols_array().as_ref());
        transforms_data.extend(self.position.extend(0.0).as_ref());
        transforms_data.extend(self.projection().to_cols_array().as_ref());
        transforms_data
    }

    pub fn change_angle(&mut self, dx: f32, dy: f32) {
        self.x_angle = (self.x_angle + dx.to_radians()) % (2.0 * std::f32::consts::PI);
        self.y_angle = (self.y_angle + dy.to_radians())
            .min(80f32.to_radians())
            .max((-80f32).to_radians());

        let cosx = self.x_angle.cos();
        let sinx = self.x_angle.sin();
        let cosy = self.y_angle.cos();
        let siny = self.y_angle.sin();
        self.forward = glam::vec3(sinx * cosy, -siny, -cosx * cosy).normalize();
    }

    /// Move in camera space
    pub fn move_pos(&mut self, units: f32) {
        self.position += units * self.forward;
    }
}

/// Acts as an [`EventHandler`] and provides control for
/// a [`Camera`]. Also maintains a [`ManagedBuffer`]
/// of column-major view matrix, 3D vector
/// camera position, and a column-major projection matrix.
/// And provides a method to update it if necessary.
///
/// [`Camera`]: struct.Camera.html
/// [`ManagedBuffer`]: ../buffer/struct.ManagedBuffer.html
/// [`EventHandler`]: ../event_handler/trait.EventHandler.html
pub struct CameraController {
    camera: Camera,
    speed: f32,
    delta_time: f32,
    buffer: ManagedBuffer,
    needs_update: bool,
}

impl CameraController {
    pub fn new(device: &wgpu::Device, camera: Camera) -> Self {
        let speed = 0.1;
        let delta_time = 0f32;
        let buffer = ManagedBuffer::from_data(
            device,
            wgpu::BufferUsage::UNIFORM,
            &camera.transforms_data(),
        );
        let needs_update = false;

        Self {
            camera,
            speed,
            delta_time,
            buffer,
            needs_update,
        }
    }

    pub fn buffer(&self) -> &ManagedBuffer {
        &self.buffer
    }

    pub fn change_speed(&mut self, change: f32) {
        self.speed += change;
    }

    pub fn set_delta_time(&mut self, delta_time: f32) {
        self.delta_time = delta_time;
    }

    pub async fn update(&mut self) {
        if self.needs_update {
            self.needs_update = false;

            self.buffer
                .update_data(0, &self.camera.transforms_data())
                .await
                .unwrap();
        }
    }
}

impl EventHandler for CameraController {
    fn handle_key(&mut self, key_input: KeyboardInput) {
        self.needs_update = true;

        match key_input.scancode {
            0x11 => self.camera.move_pos(self.speed),
            0x1F => self.camera.move_pos(-self.speed),
            _ => {}
        }
    }

    fn handle_mouse_move(&mut self, dx: f32, dy: f32) {
        self.needs_update = true;

        self.camera.change_angle(dx, dy);
    }
}
