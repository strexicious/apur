use super::math::BBox;
use super::renderer::buffer::ManagedBuffer;

pub struct DirectionalLight {
    direction: glam::Vec3,
    buffer: ManagedBuffer,
}

impl DirectionalLight {
    pub fn new<T: Into<glam::Vec3>>(device: &wgpu::Device, dir: T, scene_bb: BBox) -> Self {
        let direction = dir.into().normalize();
        let (x, y, z) = direction.into();

        let center = scene_bb.min() + (scene_bb.max() - scene_bb.min()) / 2.0;
        let scene_sphere_rad = (scene_bb.max() - scene_bb.min()).length() / 2.0;

        let eye = center - direction * scene_sphere_rad;

        let up = if x == 0.0 && z == 0.0 {
            glam::vec3(1.0, 0.0, 0.0)
        } else {
            glam::vec3(0.0, 1.0, 0.0)
        };

        let view = glam::Mat4::look_at_rh(eye, center, up);
        let projection = glam::Mat4::orthographic_rh(
            -scene_sphere_rad,
            scene_sphere_rad,
            -scene_sphere_rad,
            scene_sphere_rad,
            0.0,
            2.0 * scene_sphere_rad,
        );
        let light_mat = projection * view;

        let mut uniform_data: Vec<f32> = vec![x, y, z, 0.0];
        uniform_data.extend_from_slice(light_mat.to_cols_array().as_ref());

        let buffer = ManagedBuffer::from_data(device, wgpu::BufferUsage::UNIFORM, &uniform_data);

        Self { direction, buffer }
    }

    pub fn uniform_buffer(&self) -> &ManagedBuffer {
        &self.buffer
    }
}
