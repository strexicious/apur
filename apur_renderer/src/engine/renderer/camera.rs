use glam::{Vec3, Mat4};

pub struct Camera {
    position: Vec3,
    forward: Vec3,
    x_angle: f32,
    y_angle: f32,
}

impl Camera {
    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.forward, glam::vec3(0.0, 1.0, 0.0))
    }

    pub fn change_angle(&mut self, dx: f32, dy: f32) {
        self.x_angle = (self.x_angle + dx.to_radians()) % (2.0 * std::f32::consts::PI);
        self.y_angle = (self.y_angle + dy.to_radians()).min(80f32.to_radians()).max((-80f32).to_radians());
        
        let cosx = self.x_angle.cos();
        let sinx = self.x_angle.sin();
        let cosy = self.y_angle.cos();
        let siny = self.y_angle.sin();
        self.forward = glam::vec3(sinx * cosy, -siny, -cosx * cosy).normalize();
    }
    
    pub fn move_pos(&mut self, units: f32) {
        self.position += units * self.forward;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            forward: -Vec3::unit_z(),
            x_angle: 0.0,
            y_angle: 0.0,
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
            znear: 0.01,
            zfar: 100.0,
        }
    }

    pub fn projection(&self) -> Mat4 {
        // this implementation is meant for Vulkan NDC
        let proj = Mat4::perspective_rh_gl(self.fov_y, self.aspect_ratio, self.znear, self.zfar);
        
        let gl2vul_mat = Mat4::from_cols_array(&[
            1.0,  0.0, 0.0, 0.0,
            0.0, -1.0, 0.0, 0.0,
            0.0,  0.0, 0.5, 0.5,
            0.0,  0.0, 0.0, 1.0,
        ]);
        
        gl2vul_mat * proj
    }
}
