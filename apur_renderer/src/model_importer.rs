use super::mesh::Mesh;

#[derive(Default)]
pub struct ModelImporter;

impl ModelImporter {
    pub fn load_meshes_obj(device: &wgpu::Device, name: &str) -> Vec<Mesh> {
        let (models, _mats) = tobj::load_obj(format!("res/models/{}.obj", name).as_ref()).expect("Failed to load the model");
        
        models.into_iter().map(|m| {
            let vs = m.mesh.positions;
            let ns = m.mesh.normals;

            assert_eq!(vs.len() / 3, ns.len() / 3, "positions and normals length not same");
            
            let mut vertices = Vec::with_capacity(vs.len() + ns.len());

            vs.chunks(3).zip(ns.chunks(3)).for_each(|(vs, ns)| {
                vertices.extend(vs);
                vertices.extend(ns);
            });
            
            Mesh::new(device, &vertices, &m.mesh.indices)
        }).collect()
    }
}
