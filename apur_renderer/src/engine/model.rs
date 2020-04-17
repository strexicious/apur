use std::time::{Instant, Duration};
use zerocopy::{AsBytes};
use super::material::{MaterialManager, Material};
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
            let vs = m.mesh.positions;
            let ts = m.mesh.texcoords;
            let ns = m.mesh.normals;

            let mat_idx = {
                let mat_idx = m.mesh.material_id;
                if mat_idx.is_none() {
                    println!("no material associated, skipping model");
                    continue;
                }

                mat_idx.unwrap()
            };
            let mat_name = mats[mat_idx].name.clone();

            let needs_tcoords = if let Material::FA(_) = mat_manager.get_material(&mat_name).unwrap() { false } else { true };

            assert_eq!(vs.len() / 3, ns.len() / 3, "positions and normals length not same");
            assert!(!needs_tcoords || (vs.len() / 3 == ts.len() / 2), "not enough texture coords!");
         
            let mut vertices = Vec::with_capacity(vs.len() + ns.len() + ts.len());
            
            if needs_tcoords {
                vs.chunks(3).zip(ts.chunks(2)).zip(ns.chunks(3)).for_each(|((vs, ts), ns)| {
                    vertices.extend(vs);
                    vertices.extend(ts);
                    vertices.extend(ns);
                });
            } else {
                vs.chunks(3).zip(ns.chunks(3)).for_each(|(vs, ns)| {
                    vertices.extend(vs);
                    vertices.extend(ns);
                });
            };
            
            let indices_buffer = ManagedBuffer::from_u32_data(device, wgpu::BufferUsage::INDEX, &m.mesh.indices);
            let vertex_buffer = ManagedBuffer::from_f32_data(device, wgpu::BufferUsage::VERTEX, &vertices);

            let indices_count = m.mesh.indices.len() as u32;
            meshes.push(Mesh {
                indices_buffer,
                vertex_buffer,
                indices_count,
                mat_name,
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
