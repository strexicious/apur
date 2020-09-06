use log::trace;
use wgpu::util::DeviceExt;
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
    usage: wgpu::BufferUsage,
}

impl ManagedBuffer {
    pub fn new<T: AsBytes>(
        device: &wgpu::Device,
        usage: wgpu::BufferUsage,
        len: usize,
        mapped: bool,
    ) -> Self {
        let size_bytes = std::mem::size_of::<T>() * len;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_bytes as u64,
            usage,
            mapped_at_creation: mapped,
        });

        Self {
            buffer,
            size_bytes,
            usage,
        }
    }

    pub fn from_data<T: AsBytes>(
        device: &wgpu::Device,
        usage: wgpu::BufferUsage,
        data: &[T],
    ) -> Self {
        let byte_data = data.as_bytes();
        let size_bytes = byte_data.len();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: byte_data,
            usage: usage,
        });

        trace!("created buffer data {:?}", byte_data);

        Self {
            buffer,
            size_bytes,
            usage,
        }
    }

    pub fn write_data<T: AsBytes>(
        &mut self,
        queue: &wgpu::Queue,
        offset: usize,
        data: &[T],
    ) -> apur_error::Result<()> {
        if (data.len() * std::mem::size_of::<T>()) > (self.size_bytes - offset) {
            return Err(apur_error::APURRendererError::BufferDataSizeMismatch);
        }

        if !self.usage.contains(wgpu::BufferUsage::COPY_DST) {
            return Err(apur_error::APURRendererError::BufferUsageNotCopyDst);
        }

        queue.write_buffer(&self.buffer, offset as u64, data.as_bytes());

        trace!("wrote data {:?}", data.as_bytes());

        Ok(())
    }

    /// If MAPPABLE_PRIMARY_BUFFERS device feature is not enabled, then it's only
    /// possible to create buffers for staging on the CPU to be MAP_READ which
    /// serve to also be a COPY_DST for the GPU data buffers.
    pub async fn read_data<T: FromBytes + Copy>(&mut self) -> apur_error::Result<Vec<T>> {
        if !self.usage.contains(wgpu::BufferUsage::MAP_READ) {
            return Err(apur_error::APURRendererError::BufferUsageNotMapRead);
        }

        let buf_slice = self.buffer.slice(..);
        buf_slice
            .map_async(wgpu::MapMode::Read)
            .await
            .expect("failed to map_read from the buffer");

        trace!("read data {:?}", &*buf_slice.get_mapped_range());

        LayoutVerified::new_slice(&*buf_slice.get_mapped_range())
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
