use crate::renderer::buffer::ManagedBuffer;

/// Contains a single position component 3-Floats buffer.
pub struct UncoloredTriangle {
    buffer: ManagedBuffer,
}

impl UncoloredTriangle {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices: Vec<f32> = vec![0.0, 0.5, 0.0, 0.5, -0.5, 0.0, -0.5, -0.5, 0.0];
        let buffer = ManagedBuffer::from_data(device, wgpu::BufferUsage::VERTEX, &vertices);

        Self { buffer }
    }

    pub fn vertex_buffer(&self) -> &ManagedBuffer {
        &self.buffer
    }
}

/// Contains a single position component 3-Floats buffer.
pub struct UncoloredCube {
    buffer: ManagedBuffer,
}

impl UncoloredCube {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices: Vec<f32> = vec![
            -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
            0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5,
            -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5,
            -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
        ];
        let buffer = ManagedBuffer::from_data(device, wgpu::BufferUsage::VERTEX, &vertices);

        Self { buffer }
    }

    pub fn vertex_buffer(&self) -> &ManagedBuffer {
        &self.buffer
    }
}
