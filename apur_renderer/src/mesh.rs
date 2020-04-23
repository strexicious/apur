use super::buffer::ManagedBuffer;

pub struct Mesh {
    vertex_buffer: ManagedBuffer,
    index_buffer: ManagedBuffer,
    index_count: u32,
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertices: &[f32], indices: &[u32]) -> Self {
        for i in indices {
            assert!((*i as usize) < vertices.len(), "vertex index out of range!");
        }
        
        let vertex_buffer = ManagedBuffer::from_data(device, wgpu::BufferUsage::VERTEX, vertices);
        let index_buffer = ManagedBuffer::from_data(device, wgpu::BufferUsage::INDEX, indices);

        Self { vertex_buffer, index_buffer, index_count: indices.len() as u32 }
    }
    
    pub fn get_index_count(&self) -> u32 {
        self.index_count
    }

    pub fn get_index_buffer(&self) -> &ManagedBuffer {
        &self.index_buffer
    }

    pub fn get_vertex_buffer(&self) -> &ManagedBuffer {
        &self.vertex_buffer
    }
}