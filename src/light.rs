use super::renderer::buffer::ManagedBuffer;
use super::math::BBox;

pub struct DirectionalLight {
    direction: glam::Vec3,
    buffer: ManagedBuffer,
}

impl DirectionalLight {
    pub fn new<T: Into<glam::Vec3>>(device: &wgpu::Device, dir: T, bbox: BBox) -> Self {
        let direction = dir.into().normalize();
        let (x, y, z) = direction.into();

        let buffer = ManagedBuffer::from_data(device, wgpu::BufferUsage::UNIFORM, &[x, y, z, 0.0]);

        Self { direction, buffer }
    }

    pub fn uniform_buffer(&self) -> &ManagedBuffer {
        &self.buffer
    }
}
