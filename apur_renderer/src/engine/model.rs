use zerocopy::{AsBytes};
use super::material::MaterialManager;
use super::renderer::Renderer;
use super::buffer::ManagedBuffer;

pub struct Mesh {
    vertex_buffer: ManagedBuffer,
    indices_buffer: ManagedBuffer,
    indices_count: u32,
    mat_name: String,
    // transform: Mat4,
}

impl Mesh {
    pub fn get_mat_name(&self) -> &str {
        &self.mat_name
    }

    pub fn get_indices_count(&self) -> u32 {
        self.indices_count
    }

    pub fn get_indices_buffer(&self) -> &ManagedBuffer {
        &self.indices_buffer
    }

    pub fn get_vertex_buffer(&self) -> &ManagedBuffer {
        &self.vertex_buffer
    }
}

pub struct Scene;

impl Scene {
    pub fn load_from_obj(
        &mut self,
        device: &wgpu::Device,
        cmd_encoder: &mut wgpu::CommandEncoder,
        obj_filename: &str,
        mat_manager: &mut MaterialManager,
        renderer: &mut Renderer,
    ) {
        let (models, mats) = tobj::load_obj(format!("res/models/{}.obj", obj_filename).as_ref()).expect("Failed to load the model");
        for mat in mats.iter() {
            mat_manager.add_material(device, cmd_encoder, mat);
        }
        
        let mut meshes = vec![];
        
        for m in models.into_iter() {
            let mut indices = vec![];
            let mut vertices: Vec<f32> = vec![];

            let vs = m.mesh.positions;
            let ts = m.mesh.texcoords;
            let ns = m.mesh.normals;

            assert_eq!(vs.len() / 3, ns.len() / 3, "positions and normals length not same");

            for i in 0..(vs.len() / 3) {
                vertices.extend([vs[i*3], vs[i*3+1], vs[i*3+2]].into_iter());
                
                if vs.len() / 3 == ts.len() / 2 {
                    vertices.extend([ts[i*2], ts[i*2+1]].into_iter());
                }

                vertices.extend([ns[i*3], ns[i*3+1], ns[i*3+2]].into_iter());
            }

            let indices_count = m.mesh.indices.len() as u32;
            
            indices.extend(m.mesh.indices);
        
            let mat_idx = m.mesh.material_id.expect("no material associated");
            
            let indices_buffer = ManagedBuffer::from_u32_data(device, wgpu::BufferUsage::INDEX, &indices);
            let vertex_buffer = ManagedBuffer::from_f32_data(device, wgpu::BufferUsage::VERTEX, &vertices);

            meshes.push(Mesh {
                indices_buffer,
                vertex_buffer,
                indices_count,
                mat_name: mats[mat_idx].name.clone(),
            });
        }
        
        renderer.add_meshes(meshes, mat_manager);
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self
    }
}
