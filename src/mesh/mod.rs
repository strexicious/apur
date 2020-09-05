use super::renderer::buffer::ManagedBuffer;
use super::math::BBox;

pub mod prefabs;

/// Basically a bunch of component data buffers.
pub struct Mesh {
    positions: ManagedBuffer,
    normals: Option<ManagedBuffer>,
    texcoords: Option<ManagedBuffer>,
    indices: ManagedBuffer,
}

impl Mesh {
    pub fn positions_buffer(&self) -> &ManagedBuffer {
        &self.positions
    }

    pub fn normals_buffer(&self) -> Option<&ManagedBuffer> {
        self.normals.as_ref()
    }

    pub fn texcoords_buffer(&self) -> Option<&ManagedBuffer> {
        self.texcoords.as_ref()
    }

    pub fn indices_buffer(&self) -> &ManagedBuffer {
        &self.indices
    }
}

pub struct Model {
    bound_box: BBox,
    meshes: Vec<Mesh>,
}

impl Model {
    pub fn load(device: &wgpu::Device, obj_filename: &str) -> Model {
        let (meshes, _) = tobj::load_obj(format!("res/models/{}.obj", obj_filename), true)
            .expect("Failed to load the model");
        
        let points = meshes.iter().flat_map(|m| {
            m.mesh.positions.chunks_exact(3).map(|c| glam::vec3(c[0], c[1], c[2]))
        });
    
        let meshes = meshes
            .iter()
            .map(|m| {
                let positions =
                    ManagedBuffer::from_data(device, wgpu::BufferUsage::VERTEX, &m.mesh.positions);
                let indices =
                    ManagedBuffer::from_data(device, wgpu::BufferUsage::INDEX, &m.mesh.indices);
    
                let normals = if !m.mesh.normals.is_empty() {
                    Some(ManagedBuffer::from_data(
                        device,
                        wgpu::BufferUsage::VERTEX,
                        &m.mesh.normals,
                    ))
                } else {
                    None
                };
    
                let texcoords = if !m.mesh.texcoords.is_empty() {
                    Some(ManagedBuffer::from_data(
                        device,
                        wgpu::BufferUsage::VERTEX,
                        &m.mesh.texcoords,
                    ))
                } else {
                    None
                };
    
                Mesh {
                    positions,
                    normals,
                    texcoords,
                    indices,
                }
            })
            .collect();

        let bound_box = BBox::from_points(points);
        
        Self { bound_box, meshes }
    }

    pub fn bounding_box(&self) -> BBox {
        self.bound_box
    }

    pub fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }
}
