use zerocopy::AsBytes;

pub struct ManagedBuffer {
    // size of the element, not in bytes
    size: usize,
    buffer: wgpu::Buffer,
}

impl ManagedBuffer {
    fn from_u32_data(device: &wgpu::Device, usage: wgpu::BufferUsage, data: &[u32]) -> Self {
        let buffer = device.create_buffer_mapped(&wgpu::BufferDescriptor {
            label: None,
            size: (4 * data.len()) as u64,
            usage,
        });

        for i in 0..data.len() {
            buffer.data[i*4..(i+1)*4].copy_from_slice(data[i].as_bytes());
        }

        Self {
            size: data.len(),
            buffer: buffer.finish(),
        }
    }

    fn from_f32_data(device: &wgpu::Device, usage: wgpu::BufferUsage, data: &[f32]) -> Self {
        let buffer = device.create_buffer_mapped(&wgpu::BufferDescriptor {
            label: None,
            size: (4 * data.len()) as u64,
            usage,
        });

        for i in 0..data.len() {
            buffer.data[i*4..(i+1)*4].copy_from_slice(data[i].as_bytes());
        }

        Self {
            size: data.len(),
            buffer: buffer.finish(),
        }
    }

    fn update_u32_data(&mut self, offset: usize, data: &[u32]) {
        let mut buffer_map = pollster::block_on(
            self.buffer.map_write(4 * offset as u64, 4 * data.len() as u64)
        ).expect("error mapping buffer for write");
        let buffer = buffer_map.as_slice();

        for i in 0..data.len() {
            buffer[i*4..(i+1)*4].copy_from_slice(data[i].as_bytes());
        }
    }

    fn update_f32_data(&mut self, offset: usize, data: &[f32]) {
        let mut buffer_map = pollster::block_on(
            self.buffer.map_write(4 * offset as u64, 4 * data.len() as u64)
        ).expect("error mapping buffer for write");
        let buffer = buffer_map.as_slice();

        for i in 0..data.len() {
            buffer[i*4..(i+1)*4].copy_from_slice(data[i].as_bytes());
        }
    }
}
