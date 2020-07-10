use log::trace;
use zerocopy::{AsBytes, FromBytes, LayoutVerified};

use super::error as apur_error;

/// A data buffer on the GPU. Provides easy to use API
/// to create and manage the underlying buffer. There are
/// no restrictions of what type of data needs to be passed
/// in, as long as it implements [`AsBytes`].
///
/// [`AsBytes`]: https://docs.rs/zerocopy/0.3.0/zerocopy/trait.AsBytes.html
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
        let buffer = device.create_buffer_with_data(
            byte_data.as_slice(),
            usage | wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::MAP_READ,
        );

        Self {
            buffer,
            size_bytes: byte_data.len(),
        }
    }

    /// # Panics
    /// If data is too big to fit in the buffer at the offset provided.
    pub async fn update_data<T: AsBytes>(
        &mut self,
        offset: usize,
        data: &[T],
    ) -> apur_error::Result<()> {
        if (data.len() * std::mem::size_of::<T>()) > (self.size_bytes - offset) {
            return Err(apur_error::APURRendererError::BufferDataSizeMismatch);
        }

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
            .await
            .expect("failed to map_write into the buffer");
        buf_map.as_slice().copy_from_slice(byte_data.as_slice());

        trace!("Wrote to a mapped buffer {:?}", byte_data);

        Ok(())
    }

    pub async fn read_data<T: FromBytes + Copy>(&mut self) -> apur_error::Result<Vec<T>> {
        let buf_map = self
            .buffer
            .map_read(0, self.size_bytes as wgpu::BufferAddress)
            .await
            .expect("failed to map_read from the buffer");
        LayoutVerified::new_slice(buf_map.as_slice())
            .ok_or(apur_error::APURRendererError::BufferTypeInterpretationFailed)
            .map(|l| l.into_slice().to_owned())
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }
}

impl AsRef<wgpu::Buffer> for ManagedBuffer {
    fn as_ref(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
