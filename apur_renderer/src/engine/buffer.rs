use zerocopy::AsBytes;

pub struct ManagedBuffer {
    // size of the element, not in bytes
    size: usize,
    bytes_per_el: usize,
    buffer: wgpu::Buffer,
}

impl ManagedBuffer {
    pub fn from_u32_data(device: &wgpu::Device, usage: wgpu::BufferUsage, data: &[u32]) -> Self {
        let byte_data = data
            .iter()
            .map(|el| el.as_bytes().to_vec())
            .flatten()
            .collect::<Vec<_>>();

        let buffer = device.create_buffer_with_data(byte_data.as_slice(), usage);

        Self {
            buffer: buffer,
            size: data.len(),
            bytes_per_el: 4,
        }
    }

    pub fn from_f32_data(device: &wgpu::Device, usage: wgpu::BufferUsage, data: &[f32]) -> Self {
        let byte_data = data
            .iter()
            .map(|el| el.as_bytes().to_vec())
            .flatten()
            .collect::<Vec<_>>();

        let buffer = device.create_buffer_with_data(byte_data.as_slice(), usage);

        Self {
            buffer: buffer,
            size: data.len(),
            bytes_per_el: 4,
        }
    }

    pub fn update_u32_data(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, offset: usize, data: &[u32]) {
        // let mut buffer_map = pollster::block_on(self.buffer.map_write(4 * offset as u64, 4 * data.len() as u64)).expect("error mapping buffer for write");
        // let buffer = buffer_map.as_slice();

        // buffer.copy_from_slice(data.iter().map(|el| el.as_bytes().to_vec()).flatten().collect::<Vec<_>>().as_slice());
        
        let byte_data = data
            .iter()
            .map(|el| el.as_bytes().to_vec())
            .flatten()
            .collect::<Vec<_>>();

        let temp_buffer = device.create_buffer_with_data(byte_data.as_slice(), wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(&temp_buffer, 0, &self.buffer, 0, byte_data.len() as u64);
    }

    pub fn update_f32_data(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, offset: usize, data: &[f32]) {
        // buffer mapping is async, we need to look more into this
        // buffer mapping is requested and completed after device.poll is called
        // so this is like for multithreading or advanced usage
        // right now we can just use small buffers to copy
        
        // let mut buffer_map = pollster::block_on(self.buffer.map_write(4 * offset as u64, 4 * data.len() as u64)).expect("error mapping buffer for write");
        // let buffer = buffer_map.as_slice();

        // buffer.copy_from_slice(data.iter().map(|el| el.as_bytes().to_vec()).flatten().collect::<Vec<_>>().as_slice());

        let byte_data = data
            .iter()
            .map(|el| el.as_bytes().to_vec())
            .flatten()
            .collect::<Vec<_>>();

        let temp_buffer = device.create_buffer_with_data(byte_data.as_slice(), wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(&temp_buffer, 0, &self.buffer, 0, byte_data.len() as u64);
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_size_bytes(&self) -> usize {
        self.bytes_per_el * self.size
    }
}
