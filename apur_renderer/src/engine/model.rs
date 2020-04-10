use super::material::MaterialManager;

pub struct Mesh {
    vertex_byte_offset: u32,
    indices_byte_offset: u32,
    indices_count: u32,
    mat_name: String,
    // transform: Mat4,
}

pub struct Model {
    vertex_buffer: wgpu::Buffer,
    indices_buffer: wgpu::Buffer,
    meshes: Vec<Mesh>,
}

impl Model {
    
    pub fn load_from_obj(
        device: &wgpu::Device,
        cmd_encoder: &wgpu::CommandEncoder,
        obj_filename: &str,
        mat_manager: MaterialManager,
    ) -> Self {
        let (models, mats) = tobj::load_obj(format!("res/models/{}.obj", obj_filename).as_ref()).expect("Failed to load the model");
        
        
        let mut indices = vec![];
        let mut vertices = vec![];
        let mut meshes = vec![];
        
        for m in models.into_iter() {
            let vs = m.mesh.positions;
            let ts = m.mesh.texcoords;
            let ns = m.mesh.normals;

            assert_eq!(vs.len() / 3, ns.len() / 3, "positions and normals length not same");

            let vertex_byte_offset = vertices.len() as u32 * 4;
            for i in 0..vs.len() {
                vertices.extend([vs[i*3], vs[i*3+1], vs[i*3+2]].into_iter());
                
                if vs.len() / 3 == ts.len() / 2 {
                    vertices.extend([ts[i*2], ts[i*2+1]].into_iter());
                }

                vertices.extend([ns[i*3], ns[i*3+1], ns[i*3+2]].into_iter());
            }

            let indices_byte_offset = indices.len() as u32 * 4;
            let indices_count = m.mesh.indices.len() as u32;
            
            indices.extend(m.mesh.indices);
        
            let mat_idx = m.mesh.material_id.expect("no material associated");
            
            meshes.push(Mesh {
                vertex_byte_offset,
                indices_byte_offset,
                indices_count,
                mat_name: mats[mat_idx].name,
            });
        }

        let indices_buffer = device.create_buffer_with_data(data: &[u8], usage: BufferUsage);
        let vertex_buffer = device.create_buffer_with_data(vertices.as_slice(), wgpu::BufferUsage::VERTEX);
        
        Self { indices_buffer, vertex_buffer, meshes }
    }

    pub fn get_indices_buffer(&self) -> &wgpu::Buffer {
        &self.indices_buffer
    }

    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn get_meshes(&self) -> &[Mesh] {
        &self.meshes
    }
}