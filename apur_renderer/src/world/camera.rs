use glam::{Vec3, Mat4};

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
            }
        }
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.forward, glam::vec3(0.0, 1.0, 0.0))
    }

    pub fn projection(&self) -> Mat4 {
        let f = 1.0 / (self.frustum.fov_y.to_radians() * 0.5).tan();
        let a = f / self.frustum.aspect_ratio;
        let b = self.frustum.zfar / (self.frustum.znear - self.frustum.zfar);
        let c = self.frustum.znear * b;
        
        Mat4::from_cols(
            glam::vec4(  a, 0.0, 0.0,  0.0),
            glam::vec4(0.0,   f, 0.0,  0.0),
            glam::vec4(0.0, 0.0,   b, -1.0),
            glam::vec4(0.0, 0.0,   c,  0.0),
        )
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
