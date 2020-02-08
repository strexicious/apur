use glam::{Vec3, Mat4};

pub struct Camera {
    position: Vec3,
    forward: Vec3,
}

impl Camera {
    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.forward, glam::vec3(0.0, 1.0, 0.0))
    }
}

impl Camera {
    pub fn move_pos(&mut self, units: f32) {
        self.position += units * self.forward;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            forward: -Vec3::unit_z(),
        }
    }
}

pub struct Frustum {
    fov_y: f32,
    aspect_ratio: f32,
    znear: f32,
    zfar: f32,
}

impl Frustum {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            fov_y: 90.0,
            aspect_ratio: width as f32 / height as f32,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn projection(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov_y, self.aspect_ratio, self.znear, self.zfar)
    }
}
