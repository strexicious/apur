use apur_renderer::buffer::ManagedBuffer;

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

pub fn load_model(device: &wgpu::Device, obj_filename: &str) -> Vec<Mesh> {
    let (models, _) = tobj::load_obj(format!("res/models/{}.obj", obj_filename), true)
        .expect("Failed to load the model");

    models
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
        .collect()
}
