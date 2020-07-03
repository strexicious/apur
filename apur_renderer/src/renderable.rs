use super::buffer::ManagedBuffer;
use super::texture::Texture2D;

pub struct IndexedVertex {
    vertex_buffer: ManagedBuffer,
    index_buffer: ManagedBuffer,
    index_count: u32,
}

impl IndexedVertex {
    pub fn from_indexed_vertices(device: &wgpu::Device, vertices: &[f32], indices: &[u32]) -> Self {
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

pub enum Geometry {
    Solid(IndexedVertex), // vertices only have positions
    TexturedSolid(IndexedVertex), // vertices have positions and texture coords
    // water
    // cloth
    // etc
}

pub enum Material {
    // just to have something, we put this here
    DiffuseColor(f32, f32, f32, f32),
    SpecularColor(f32, f32, f32, f32),
    DiffuseMapped(Texture2D),
    SpecularMapped(Texture2D),
    Mixed(Box<Material>, Box<Material>, f32),
}

pub struct Renderable {
    geometry: Geometry,
    material: Material,
}
