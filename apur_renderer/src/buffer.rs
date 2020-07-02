use zerocopy::AsBytes;

pub struct ManagedBuffer {
    size_bytes: usize,
    buffer: wgpu::Buffer,
}

impl ManagedBuffer {
    pub fn from_data<T: AsBytes>(
        device: &wgpu::Device,
        usage: wgpu::BufferUsage,
        data: &[T],
    ) -> Self {
        let byte_data = data
            .iter()
            .flat_map(|el| el.as_bytes().to_vec())
            .collect::<Vec<_>>();
        let buffer = device
            .create_buffer_with_data(byte_data.as_slice(), usage | wgpu::BufferUsage::MAP_WRITE);

        Self {
            buffer,
            size_bytes: byte_data.len(),
        }
    }

    pub async fn update_data<T: AsBytes>(
        &mut self,
        offset: usize,
        data: &[T],
    ) -> Result<(), wgpu::BufferAsyncErr> {
        assert!(
            (data.len() * std::mem::size_of::<T>()) <= (self.size_bytes - offset),
            "data does not fit into the buffer!"
        );

        let byte_data = data
            .iter()
            .flat_map(|el| el.as_bytes().to_vec())
            .collect::<Vec<_>>();
        let mut buf_map = self
            .buffer
            .map_write(
                offset as wgpu::BufferAddress,
                byte_data.len() as wgpu::BufferAddress,
            )
            .await?;
        buf_map.as_slice().copy_from_slice(byte_data.as_slice());
        Ok(())
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn get_size_bytes(&self) -> usize {
        self.size_bytes
    }
}
