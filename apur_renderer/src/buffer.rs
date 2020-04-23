use zerocopy::AsBytes;

pub struct ManagedBuffer {
    size_bytes: usize,
    buffer: wgpu::Buffer,
}

impl ManagedBuffer {
    pub fn from_data<T: AsBytes>(device: &wgpu::Device, usage: wgpu::BufferUsage, data: &[T]) -> Self {
        let byte_data = data
            .iter()
            .map(|el| el.as_bytes().to_vec())
            .flatten()
            .collect::<Vec<_>>();

        let buffer = device.create_buffer_with_data(byte_data.as_slice(), usage | wgpu::BufferUsage::COPY_DST);

        Self { buffer, size_bytes: byte_data.len() }
    }

    pub fn update_data<T: AsBytes>(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, offset: usize, data: &[T]) {
        assert!((data.len() * std::mem::size_of::<T>()) <= (self.size_bytes - offset), "data does not fit into the buffer!");
        
        let byte_data = data
            .iter()
            .map(|el| el.as_bytes().to_vec())
            .flatten()
            .collect::<Vec<_>>();

        let temp_buffer = device.create_buffer_with_data(byte_data.as_slice(), wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(&temp_buffer, 0, &self.buffer, offset as u64, byte_data.len() as u64);
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn get_size_bytes(&self) -> usize {
        self.size_bytes
    }
}